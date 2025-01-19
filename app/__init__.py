from flask import Flask
from flask_socketio import SocketIO
import os
import json

# Initialize the SocketIO instance with CORS allowed from any origin
socketio = SocketIO(cors_allowed_origins="*")

# Function to create and configure the Flask app
def create_app():

    # Load configuration settings from the config.json file
    with open('config.json', 'r') as file:
        config = json.load(file)
    
    # Create the Flask app and set a secret key for session management
    app = Flask(__name__)
    app.config['SECRET_KEY'] = os.urandom(24)
    
    # Initialize the SocketIO instance with the Flask app
    socketio.init_app(app)
    
    # Import and initialize routes with the app, SocketIO instance, and configuration settings
    from .routes import init_routes
    init_routes(app, socketio, config)
    
    return app
