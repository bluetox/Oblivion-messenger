from flask import jsonify, request, render_template
from flask_socketio import join_room, emit, leave_room
from databases.database import Database
import secrets

# Dictionary to map SocketIO session IDs to unique session identifiers (e.g., {sid: session_id})
session_ids = {}

# Dictionary to map session identifiers to user IDs (e.g., {session_id: user_id})
user_ids = {}

# Initialize the database instance
db = Database()
db.create_database()

def init_routes(app, socketio, config):
    # Route to render the landing page HTML from the templates folder
    @app.route('/')
    def home():
        return render_template('landing.html')
    
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
    
    @app.route('/settings')
    def settings():
        return render_template('settings.html')
    
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
        if not user_id:
            return jsonify({"error" : "Could not get your session. Establishing a new one"}), 404
        # Return the user ID as JSON
        return jsonify({"user_id": user_id})

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
        
        # Verify that both username and password are provided
        if username and password:
            try:
                user_id = db.add_user(username, password)
                session = secrets.token_urlsafe(config['token_lengh'])
                user_ids[session] = user_id
                
                return jsonify({"status": "registered","token" : session}), 201
            except: 
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
            status, user_id = db.login(username, password)
            
            if status == False:
                # Return an error if the password or username is incorrect
                return jsonify({"status": "Password or username incorrect"})
            else:
                session = secrets.token_urlsafe(config['token_lengh'])
                user_ids[session] = user_id
                return jsonify({"status": f"logged_in", "token" : session})
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
    
    # Forward a message to the specified user based on their user ID
    @socketio.on('send_message')
    def handle_send_message(data):
        # Extract the target user ID from the data
        target_user_id = data.get("target_user_id")
        session_id = request.cookies.get('session_id')
        user_id = user_ids.get(session_id)
        # Check if a target user ID was provided
        if target_user_id:
            # Send the message to the room associated with the target user ID
            emit('receive_message', {
                "message": data["message"],
                "from_user_id": user_id
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
            
    @socketio.on('append_KyberKey')
    def appendKey(data):
        session_id = request.cookies.get('session_id')
        print("SESSION ID : ", session_id)
        user_id = user_ids.get(session_id)
        
        target_user_id = data.get("target_user_id")
        key = data.get("public_key")
        
        emit('append_KyberKey',{'source_id' : user_id ,'public_key' : key},room=target_user_id)
        
    @socketio.on('append_cypher')
    def appendCypher(data):
        session_id = request.cookies.get('session_id')
        user_id = user_ids.get(session_id)
        cyphertext = data.get('cypherText')
        destination = data.get('dest_id')
        
        emit('append_cypher',{'cypherText' : cyphertext, 'from_user_id' : user_id},room=destination)

        
    @socketio.on('get_status')
    def get_status(data):
        # Retrieve the list of user IDs from the client (passed as 'user_ids' in the event data)
        user_ids = data.get('allChatIds', [])
        # Retrieve the list of online user IDs by checking if they are in any active rooms
        online_user_ids = [
            user_id for user_id in user_ids
            if user_id in socketio.server.manager.rooms.get(request.namespace, {})
        ]

        # Emit the list of online users back to the client
        emit('onlines', {'online_user_ids': online_user_ids})