from flask_socketio import join_room, emit
from flask import Flask, render_template, jsonify, request
import time
from threading import Lock


# Shared data structures for managing users and connections
public_keys = {}
session_ids = {}
heartbeat_times = {}
heartbeat_lock = Lock()

def handle_register(data):
    user_id = data.get('user_id')
    public_key = data.get('public_key')
    
    if user_id and public_key:
        join_room(user_id)
        public_keys[user_id] = public_key
        session_ids[request.sid] = user_id
        print(f"User {user_id} registered and joined their room with session ID {request.sid}.")

def handle_unregister(data):
    user_id = data.get('user_id')
    if user_id and user_id in public_keys:
        sid = next((sid for sid, uid in session_ids.items() if uid == user_id), None)
        if sid:
            session_ids.pop(sid, None)
        public_keys.pop(user_id, None)
        print(f"User {user_id} unregistered successfully.")
        return jsonify({"message": f"User {user_id} unregistered successfully"}), 200
    else:
        return jsonify({"error": "User ID not found"}), 404

def handle_heartbeat():
    with heartbeat_lock:
        heartbeat_times[request.sid] = time.time()

def monitor_heartbeats():
    while True:
        time.sleep(2)
        current_time = time.time()
        with heartbeat_lock:
            to_disconnect = [sid for sid, last_time in heartbeat_times.items() if current_time - last_time > 4]
            for sid in to_disconnect:
                user_id = session_ids.pop(sid, None)
                if user_id:
                    public_keys.pop(user_id, None)
                    print(f"User ID {user_id} has been disconnected due to a missing heartbeat.")
                heartbeat_times.pop(sid, None)

def handle_send_message(data):
    target_user_id = data.get("target_user_id")
    if target_user_id:
        # Send the message to the target user's room
        emit('receive_message', {
            "message": data["message"],
            "from_user_id": data["from_user_id"]
        }, room=target_user_id)  # Send message to target user's room
        print(f"Message forwarded to {target_user_id}")
    else:
        emit('error', {'message': 'Target user ID is required for sending the message.'})