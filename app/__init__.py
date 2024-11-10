from flask import Flask
from flask_socketio import SocketIO
import os

socketio = SocketIO()

def create_app():
    app = Flask(__name__)
    app.config['SECRET_KEY'] = os.urandom(24)
    
    # Initialize SocketIO with the app
    socketio.init_app(app)
    
    # Register the routes and socket events
    from app.routes import init_routes
    init_routes(app, socketio)  # Pass socketio here
    
    return app
