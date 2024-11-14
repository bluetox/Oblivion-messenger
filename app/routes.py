from flask import jsonify, request, render_template
from flask_socketio import join_room, emit, leave_room
from app.socket import heartbeat_times
from databases.database import Database
import secrets

# Dictionary to map SocketIO session IDs to unique session identifiers (e.g., {sid: session_id})
session_ids = {}

# Dictionary to map session identifiers to user IDs (e.g., {session_id: user_id})
user_ids = {}

# Dictionary to store public keys associated with user IDs (e.g., {user_id: public_key})
public_keys = {}

# Initialize the database instance
db = Database()

def init_routes(app, socketio, config):
    # Route to render the landing page HTML from the templates folder
    @app.route('/')
    def home():
        return render_template('landing.html')
    
    @app.route('/test')
    def test():
        return render_template('test.html')
    
    # Route to render the chat page HTML
    @app.route('/chat')
    def chat():
        return render_template('chat.html')
    
    # Route to render the registration page HTML
    @app.route('/register')
    def register():
        return render_template('register.html')
    
    # Route to render the login page HTML
    @app.route('/login')
    def login():
        return render_template('login.html')
    
    # API route to create session and user IDs based on config settings
    @app.route('/api/create_cookies')
    def set_cookies():
        # Generate a session ID and user ID based on config-specified lengths
        session_id = secrets.token_urlsafe(config['token_lengh'])
        user_id = secrets.token_hex(config['user_id_size'])
        
        # Store the session and user IDs in their respective dictionaries
        user_ids[session_id] = user_id
        
        # Return the session and user IDs to the client as JSON
        return jsonify({
            "session_cookie": session_id,
            "user_id": user_id
        })

    # API route to retrieve a user ID based on a session ID and set a new public key
    # This ensures that the user retains the same user ID across sessions, while allowing for a new public key
    @app.route('/api/get-user-data')
    def get_user_data():
        # Get the session ID from the request arguments
        session_cookie = request.args.get('session_id')
        
        # Retrieve the associated user ID from the user_ids dictionary
        user_id = user_ids.get(session_cookie)
        
        # Update the public key for this user ID
        public_keys[user_id] = request.args.get('public_key')
        
        # Return the user ID as JSON
        return jsonify({"user_id": user_id})

    # API route to retrieve the public key associated with a specific user ID
    # Used for client-side RSA encryption
    @app.route('/api/get-public-key')
    def get_public_key():
        # Retrieve the user ID from the request arguments
        user_id = request.args.get('user_id')
        
        # Verify if the user exists and has an assigned public key
        if user_id and user_id in public_keys:
            
            # Return the public key as JSON
            return jsonify({"public_key": public_keys[user_id]})
        
        # Return an error message if the user ID is not found or has no public key
        return jsonify({"error": "User ID not found or public key not registered"}), 404

    
    # API route to remove session and user IDs, as well as the associated sid and public key for a specific user
    # Used for unregistering a user
    @app.route('/api/unregister')
    def handle_unregister():
        # Retrieve the session ID from cookies
        session_id = request.cookies.get('session_id')
        
        # Check if the session ID exists
        if not session_id:
            return jsonify({"status": "No active session found"}), 400

        # Find the sid associated with the session ID
        sid = None
        for current_sid, user_session_id in session_ids.items():
            if user_session_id == session_id:
                sid = current_sid
                break
        
        # Handle cases where no sid is found for the session ID
        if not sid:
            return jsonify({"status": "No sid assigned to session_id"}), 404
        
        # Remove the sid from session IDs
        del session_ids[sid]
        print(f"Session with ID {sid} removed.")
        
        # Verify if a public key is linked to the user ID and remove it if found
        user_id = user_ids.get(session_id)
        if user_id not in public_keys:
            return jsonify({"status": "No public key assigned to this user ID"}), 404
        
        # Delete the public key for the user ID
        del public_keys[user_id]
        print(f"Public key for user ID {user_id} removed.")

        # Return a success message
        return jsonify({"status": f"Unregistered user with session ID {sid}"}), 200

        # API route to create an account with a username and hashed password provided in JSON format
    @app.route('/api/create_account', methods=['POST'])
    def create_account():
        # Retrieve the incoming data as JSON
        data = request.get_json()
        
        # Extract the username and check if it already exists in the database
        username = data.get('username')
        if db.check_user_presence(username):
            # Return an error if the username is already taken
            return jsonify({"error": "Select another username"}), 409
        
        # Extract the hashed password
        password = data.get('password')
        
        # Generate a unique user ID based on the configuration settings
        user_id = secrets.token_hex(config['user_id_size'])
        
        # Verify that both username and password are provided
        if username and password:
            # Attempt to add the new user to the database
            if db.add_user(username, user_id, password):
                # Return a success message and the generated user_id
                return jsonify({"status": "registered", "user_id": user_id}), 201
            else:
                # Return an error message if there is an issue with the database operation
                return jsonify({"error": "There was an error writing to the database"}), 500
        else:
            # Return an error if either the username or password is missing
            return jsonify({"error": "Username and password are required"}), 400

    # API route to log in a user by verifying credentials against the database
    @app.route('/api/login', methods=['POST'])
    def check_login():
        # Retrieve the incoming data as JSON
        data = request.get_json()
        
        # Extract the username and password and verify if both are provided
        username = data.get('username')
        password = data.get('password')
        if username and password:
            # Check if the login process is successful
            if not db.login(username, password):
                # Return an error if the password or username is incorrect
                return jsonify({"status": "Password or username incorrect"})
            else:
                # Return a success message if the login process is successful
                return jsonify({"status": f"Logged in as {username}"})
        else:
            # Return an error if the username and/or password is missing
            return jsonify({"error": "Username and password are required"}), 400
    
    # Handle the initial socket connection
    @socketio.on('connect')
    def connect():
        try:
            # Retrieve the session ID from the request cookies and check if it exists
            session_id = request.cookies.get('session_id')
            if not session_id:
                raise ValueError("Session ID is missing in cookies.")
            
            # Get the user ID associated with the session and check if it exists
            user_id = user_ids.get(session_id)
            if not user_id:
                raise KeyError(f"User not found for session ID: {session_id}")
            
            # Log when a user joins a room
            print(f"User {user_id} registered and joined their room with session token {request.sid}.")
            
            # Join the room associated with the user ID
            join_room(user_id)
            
            # Map the session ID to the socket ID
            session_ids[request.sid] = session_id
            
        # Handle exceptions
        except ValueError as ve:
            print(f"Error: {ve}")
        except KeyError as ke:
            print(f"Error: {ke}")
        except Exception as e:
            print(f"Connection failed due to an unexpected error: {e}")
    
    # Acknowledge and store the user's public key
    @socketio.on('append_key')
    def append_key(data):
        # Retrieve the session ID from the request cookies
        session_id = request.cookies.get('session_id')
        
        # Get the user ID associated with the session
        user_id = user_ids.get(session_id)
        
        # Extract the public key from the request data
        public_key = data.get('public_key')
        
        # Assign the public key to the user ID
        public_keys[user_id] = public_key
        
        # Return the success status as JSON
        return jsonify({"status": True})
    
    # Forward a message to the specified user based on their user ID
    @socketio.on('send_message')
    def handle_send_message(data):
        # Extract the target user ID from the data
        target_user_id = data.get("target_user_id")
        
        # Check if a target user ID was provided
        if target_user_id:
            # Send the message to the room associated with the target user ID
            emit('receive_message', {
                "message": data["message"],
                "from_user_id": data["from_user_id"]
            }, room=target_user_id)
            
            # Log the message forwarding action
            print(f"Message forwarded to {target_user_id}")
        else:
            # Emit an error message if no target user ID was provided
            emit('error', {'message': 'Target user ID is required for sending the message.'})

    # Handle socket disconnection
    @socketio.on('disconnect')
    def disconnect():
        # Remove the session ID from the session tracking dictionary
        user_id = session_ids.pop(request.sid, None)
        if user_id:
            # Leave the room associated with the user ID
            leave_room(user_id)
            # Log the disconnection
            print(f"User {user_id} disconnected.")
