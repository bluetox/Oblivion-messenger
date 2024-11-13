from flask import Flask
from flask_socketio import SocketIO
import os
import json

socketio = SocketIO(cors_allowed_origins="*")

def create_app():
    with open('config.json','r') as file:
        config = json.load(file)
    app = Flask(__name__)
    app.config['SECRET_KEY'] = os.urandom(24)
    
    socketio.init_app(app)
    
    from .routes import init_routes
    init_routes(app, socketio, config)
    
    return app

