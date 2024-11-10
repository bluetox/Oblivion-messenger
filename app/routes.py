from flask import jsonify, request, render_template
from app.socket import handle_register, handle_unregister, public_keys
import secrets
from app.socket import public_keys, session_ids, heartbeat_times,handle_send_message


def init_routes(app, socketio):
    
    @app.route('/chat')
    def chat():
        return render_template('index.html')
    @app.route('/')
    def home():
        return render_template('landing.html')
    @app.route('/login')
    def login():
        return render_template('login.html')

    @app.route('/create_account', methods=['POST'])
    def create_account():
        data = request.get_json()
        username = data.get('username')
        password = data.get('password')
        if username and password:
            # Handle account creation logic here (e.g., database save)
            return jsonify({"status": "registered","hashed_password":password})
        else:
            return jsonify({"error": "Username and password are required"}), 400
    
    @app.route('/generate-id')
    def generate_id():
        user_id = secrets.token_hex(32)  # Generate a unique ID
        return jsonify({"id": user_id})

    @app.route('/get-public-key')
    def get_public_key():
        user_id = request.args.get('user_id')
        if user_id and user_id in public_keys:
            return jsonify({"public_key": public_keys[user_id]})
        else:
            return jsonify({"error": "User ID not found or public key not registered"}), 404

    @app.route('/unregister', methods=['POST'])
    def handle_unregister():
        user_id = request.json.get('user_id')
        if user_id and user_id in public_keys:
            sid = next((sid for sid, uid in session_ids.items() if uid == user_id), None)
            if sid:
                session_ids.pop(sid, None)  # Remove from session tracking
                heartbeat_times.pop(sid, None)  # Remove from heartbeat tracking
            public_keys.pop(user_id, None)  # Remove public key
            print(f"User {user_id} unregistered successfully.")
            return jsonify({"message": f"User {user_id} unregistered successfully"}), 200
        else:
            return jsonify({"error": "User ID not found"}), 404

    socketio.on_event('register', handle_register)
    socketio.on_event('unregister', handle_unregister)
    socketio.on_event('send_message', handle_send_message)
