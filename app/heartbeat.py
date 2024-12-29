import time
from threading import Lock
from app.socket import session_ids, public_keys

heartbeat_times = {}
heartbeat_lock = Lock()

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
