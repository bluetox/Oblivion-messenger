from flask import jsonify, request, render_template
from flask_socketio import join_room, emit, leave_room
from app.socket import heartbeat_times
from databases.database import Database
import secrets

user_keys = {}
public_keys = {}
session_ids = {}
user_ids = {}
db = Database()

def init_routes(app, socketio, config):
    # Handle routes rendering the html from the templates folder
    @app.route('/')
    def home():
        return render_template('landing.html')
    
    @app.route('/chat')
    def chat():
        return render_template('chat.html')
    
    @app.route('/register')
    def register():
        return render_template('register.html')
    
    @app.route('/login')
    def login():
        return render_template('login.html')
    
    @app.route('/debug')
    def debug():
        return render_template('test_features.html')

    @app.route('/api/create_cookies')
    def set_cookies():
        session_cookie = secrets.token_urlsafe(config['token_lengh'])
        user_id = secrets.token_hex(config['user_id_size'])
        user_ids[session_cookie] = user_id
        return jsonify({
            "session_cookie": session_cookie,
            "user_id": user_id
        })
        
    @app.route('/api/get-user-data')
    def get_user_data():
        session_cookie = request.args.get('session_id')
        user_id = user_ids.get(session_cookie)
        public_keys[user_id] = request.args.get('public_key')
        return jsonify({"user_id": user_id})

    @app.route('/api/get-public-key')
    def get_public_key():
        user_id = request.args.get('user_id')
        if user_id and user_id in public_keys:
            return jsonify({"public_key": public_keys[user_id]})
        else:
            return jsonify({"error": "User ID not found or public key not registered"}), 404
    
    # Remove a user_id from known
    @app.route('/api/unregister')
    def handle_unregister():
        session_id = request.cookies.get('session_id') 
        if not session_id:
            return jsonify({"status": f"You don't have any current session"})

        for sid, user_id in session_ids.items():
            if user_id == session_id:
                break
        if not sid:
            return jsonify({"No sid assigned to session_id"})
            
        del session_ids[sid]
        print(f"Session with ID {sid} removed.")
        if not user_id in public_keys:
            return jsonify({"No public key assigned to this user id"})
        del public_keys[user_id]
        
        print(f"Public key for session {sid} removed.")

        return jsonify({"status": f"unregistered {sid}"}), 200

    @app.route('/api/create_account', methods=['POST'])
    def create_account():
        data = request.get_json()
        username = data.get('username')
        if db.check_user_presence(username):
            return jsonify({"error": "Select another username"})
        password = data.get('password')
        user_id = secrets.token_hex(config['user_id_size'])
        if username and password:
            if db.add_user(username, user_id, password):
                return jsonify({"status": "registered", "session_token": user_id})
            else:
                return jsonify({"error": "There was an error writing to the database"})
        else:
            return jsonify({"error": "Username and password are required"}), 400
    
    @app.route('/api/login', methods=['POST'])
    def check_login():
        data = request.get_json()
        username = data.get('username')
        password = data.get('password')
        if username and password:
            if not db.login(username, password):
                return jsonify({"status": "Password or username wrong"})
            else:
                return jsonify({"status": f"Logged in as {username}"})
        else:
            return jsonify({"error": "Username and password are required"}), 400
    
    @socketio.on('connect')
    def connect():
        try:
            session_id = request.cookies.get('session_id')
            if not session_id:
                raise ValueError("Session ID is missing in cookies.")
            user_id = user_ids.get(session_id)
            if not user_id:
                raise KeyError(f"User not found for session ID: {session_id}")
            print(f"User {user_id} registered and joined their room with session token {request.sid}.")
            join_room(user_id)  # Join the user to their own room
            session_ids[request.sid] = user_id  # Map session id to user_id
            
        except ValueError as ve:
            print(f"Error: {ve}")
        except KeyError as ke:
            print(f"Error: {ke}")
        except Exception as e:
            print(f"Connection failed due to an unexpected error: {e}")
    
    @socketio.on('append_key')
    def append_key(data):
        session_id = request.cookies.get('session_id')
        user_id = user_ids.get(session_id)
        public_key = data.get('public_key')
        public_keys[user_id] = public_key
        return jsonify({"status": True})
    
    @socketio.on('send_message')
    def handle_send_message(data):
        target_user_id = data.get("target_user_id")
        if target_user_id:
            emit('receive_message', {
                "message": data["message"],
                "from_user_id": data["from_user_id"]
            }, room=target_user_id)
            print(f"Message forwarded to {target_user_id}")
        else:
            emit('error', {'message': 'Target user ID is required for sending the message.'})

    @socketio.on('disconnect')
    def disconnect():
        user_id = session_ids.pop(request.sid, None)
        if user_id:
            leave_room(user_id)  # Ensure the user leaves their room
            print(f"User {user_id} disconnected.")
