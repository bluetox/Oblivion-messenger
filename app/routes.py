from flask import jsonify, request, render_template
from flask_socketio import join_room, emit, leave_room
from databases.database import Database
import secrets
import base64
import re

messages_on_hold = {}
# Dictionary to map SocketIO session IDs to unique session identifiers (e.g., {sid: session_id})
session_ids = {}

# Dictionary to map session identifiers to user IDs (e.g., {session_id: user_id})
user_ids = {}

keys = {}
# Initialize the database instance
db = Database()
db.create_database()

def init_routes(app, socketio, config):
    
    def is_valid_session_id(session_id):
        
        # Match URL-safe base64 strings (e.g., A-Za-z0-9-_)
        token_length = config['token_length'] * 4 // 3  # Rough expected length
        return bool(re.fullmatch(rf"[A-Za-z0-9\-_]{{{token_length},}}", session_id))
    
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
    @app.route('/test')
    def test():
        return render_template('test.html')
    
    
    # API route to create session and user IDs based on config settings
    @app.route('/api/create_cookies')
    def set_cookies():
        # Generate a session ID and user ID based on config-specified lengths
        session_id = secrets.token_urlsafe(config['token_length'])
        user_id = secrets.token_hex(config['user_id_size'])
        
        # Store the session and user IDs in their respective dictionaries
        user_ids[session_id] = user_id
        
        # Return the session and user IDs to the client as JSON
        return jsonify({
            "session_cookie": session_id,
            "user_id": user_id
        })
        
    @app.route('/api/set_decryption_key', methods=['POST'])
    def set_decryption_key():
        
        data = request.get_json()
        
        session_id = request.args.get('session_id')
        user_id = user_ids[session_id]
        
        key = data['key']
        keys[user_id] = key
        
        return jsonify({
            "status" : "success"
        })
    
    @app.route('/api/get_decryption_key')
    def get_decryption_key():
        user_id = request.args.get('user_id')
        key = keys[user_id]
        
        return jsonify({
            "key" : key
        })
        
    # API route to retrieve a user ID based on a session ID and set a new public key
    # This ensures that the user retains the same user ID across sessions, while allowing for a new public key
    @app.route('/api/get-user-data')
    def get_user_data():
        
        # Get the session ID from the request arguments
        session_id = request.cookies.get('session_id')
        if not is_valid_session_id(session_id):
            return jsonify({"error" : "The given session id is not well formated"}), 405
        
        # Retrieve the associated user ID from the user_ids dictionary
        user_id = user_ids.get(session_id)
        if not user_id or not session_id:
            return jsonify({"error" : "Could not get your session. Establishing a new one"}), 404
        
        # Return the user ID as JSON
        return jsonify({"status" : "success", "user_id": user_id}), 200

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
                session = secrets.token_urlsafe(config['token_length'])
                user_ids[session] = user_id
                
                return jsonify({"status": "registered", "token" : session}), 201
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
                session = secrets.token_urlsafe(config['token_length'])
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
        target_user_id = data.get("target_user_id")
        signature = data.get("signature")
        session_id = request.cookies.get('session_id')
        user_id = user_ids.get(session_id)

        if not target_user_id:
            emit('error', {'message': 'Target user ID is required for sending the message.'})
            return

        if target_user_id in socketio.server.manager.rooms.get(request.namespace, {}):
            # Room exists, forward the message
            emit('receive_message', {
                "message": data["message"],
                "from_user_id": user_id,
                "signature": signature
            }, room=target_user_id)
            print(f"Message forwarded to {target_user_id}")
        else:
            # Room does not exist, store the message
            if target_user_id not in messages_on_hold:
                messages_on_hold[target_user_id] = []

            message = {
                "from_user_id" : user_id,
                "message" : data["message"] 
            }
            messages_on_hold[target_user_id].append(message)
            print(messages_on_hold)

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
        user_ids = data.get('allDestIds', [])

        # Retrieve the list of online user IDs by checking if they are in any active rooms
        online_user_ids = [
            user_id for user_id in user_ids
            if user_id in socketio.server.manager.rooms.get(request.namespace, {})
            
        ]

        # Emit the list of online users back to the client
        emit('onlines', {'online_user_ids': online_user_ids})
        
    @socketio.on('encryptedFile')
    def transfer_file(data):
        if data:
            try:
                # Access file data and file name from the received data
                file_name = data.get('fileName')
                file_data = data.get('file')
                destination = data.get('dest')
                session_id = request.cookies.get('session_id')


                #TO IMPLEMENT ABSOLUTELY
                source = user_ids.get(session_id)
                
                if not file_name or not file_data:
                    emit("error", {"message": "Invalid file data received."})
                    return

                # Decode the file_data from binary if needed (base64 encoding for transfer)
                encoded_data = base64.b64encode(file_data).decode('utf-8')

                # Emit the file name and encoded file data back to the client
                emit('receivedFile', {"file": encoded_data, "fileName": file_name},room=destination)
            except Exception as e:
                emit("error", {"message": f"Error processing file: {str(e)}"})
        else:
            emit("nope")  # Emit if no data is received



    @socketio.on('get_messages')
    def get_messages():
        session_id = request.cookies.get('session_id')
        user_id = user_ids.get(session_id)
        messages = messages_on_hold.pop(user_id, [])

        return messages
    
    @socketio.on('dilithium_key')
    def send_dilithium_key(data):
        
        session_id = request.cookies.get('session_id')
        dest_id = data.get('dest_id')
        user_id = user_ids.get(session_id)

        key = data.get("key")
        emit("append_dilithium_key", {"source_id": user_id, "key" : key},room=dest_id)
