import os
import platform
import subprocess
import sys

def install_requirements():
    """Install necessary Python packages."""
    subprocess.run([sys.executable, "-m", "pip", "install", "flask", "bcrypt", "flask_socketio", "gunicorn"])

def setup_linux():
    """Setup and run the server in a Linux environment."""
    print("Running in a Linux/Unix-like environment. Trying to launch the server.")

    subprocess.run([sys.executable, "-m", "venv", "venv"])
    
    activate_script = "./venv/bin/activate"
    install_cmd = " && ".join([
        f"source {activate_script}",
        "pip install flask bcrypt flask_socketio gunicorn"
    ])
    subprocess.run(install_cmd, shell=True, executable="/bin/bash")
    
    print("Virtual environment set up. Deactivating.")
    
    subprocess.run(["./venv/bin/python", "run.py"])

def setup_other():
    """Setup for non-Linux environments."""
    print("Not running in a Linux/Unix-like environment.")
    install_requirements()

if os.name == 'posix' or platform.system() == 'Linux':
    setup_linux()
else:
    setup_other()
