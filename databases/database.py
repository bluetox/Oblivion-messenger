import sqlite3
import bcrypt
from datetime import datetime
import secrets
import json

# Load configuration from config.json
with open('config.json', 'r') as file:
    config = json.load(file)

class Database:
    def create_database(self):
        """
        Creates the 'users' database and the 'users' table if it doesn't already exist.
        """
        try:
            conn = sqlite3.connect('databases/users.db')
            cursor = conn.cursor()
            
            # Create the 'users' table if it doesn't exist
            cursor.execute("""
                CREATE TABLE IF NOT EXISTS users (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    account_creation_timestamp TEXT NOT NULL,
                    user_id TEXT UNIQUE NOT NULL,
                    password TEXT NOT NULL,
                    username TEXT UNIQUE NOT NULL
                )
            """)
            
            conn.commit()
            conn.close()
            print("Database and table created successfully.")
        except sqlite3.Error as e:
            print(f"Database error: {e}")

    def add_user(self, username, password):
        """
        Adds a new user to the database.

        Args:
            username (str): The username of the user.
            password (str): The plaintext password of the user.

        Returns:
            str: The user_id of the created user or None if an error occurs.
        """
        try:
            user_id = secrets.token_hex(config['user_id_size'])
            conn = sqlite3.connect('databases/users.db')
            salt = bcrypt.gensalt(12)
            hashed_password = bcrypt.hashpw(password.encode('utf-8'), salt)
            cursor = conn.cursor()
            now = datetime.now().isoformat()
            
            cursor.execute("""
                INSERT INTO users (account_creation_timestamp, user_id, password, username)
                VALUES (?, ?, ?, ?)
            """, (now, user_id, hashed_password, username))
            
            conn.commit()
            conn.close()
            return user_id
        except sqlite3.IntegrityError:
            print(f"Error: Username '{username}' already exists.")
            return None
        except sqlite3.Error as e:
            print(f"Database error: {e}")
            return None

    def check_user_presence(self, username):
        """
        Checks if a user exists in the database.

        Args:
            username (str): The username to check.

        Returns:
            bool: True if the user exists, False otherwise.
        """
        try:
            conn = sqlite3.connect('databases/users.db')
            cursor = conn.cursor()
            cursor.execute("SELECT * FROM users WHERE username = ?", (username,))
            data = cursor.fetchone()
            conn.close()
            return data is not None
        except sqlite3.Error as e:
            print(f"Database error: {e}")
            return False

    def login(self, username, password):
        """
        Authenticates a user.

        Args:
            username (str): The username of the user.
            password (str): The plaintext password of the user.

        Returns:
            tuple: (bool, str) - Success status and either user_id or error message.
        """
        try:
            conn = sqlite3.connect('databases/users.db')
            cursor = conn.cursor()
            cursor.execute("SELECT password, user_id FROM users WHERE username = ?", (username,))
            data = cursor.fetchone()
            conn.close()
    
            if data:
                hashed_password, user_id = data
                if bcrypt.checkpw(password.encode('utf-8'), hashed_password):
                    return True, user_id  # Authentication successful
                else:
                    return False, "Invalid credentials"
            else:
                return False, "User not found"
    
        except sqlite3.OperationalError as op_error:
            print(f"Operational error: {op_error}")
            return False, "Database operation failed"
    
        except sqlite3.Error as db_error:
            print(f"Database error: {db_error}")
            return False, "An unexpected database error occurred"
    
        except Exception as ex:
            print(f"Unexpected error: {ex}")
            return False, "An unexpected error occurred"
    
    def load_session(self, session):
        """
        Loads a user's session.

        Args:
            session (str): The session identifier.

        Returns:
            dict: A dictionary containing user details or None if not found.
        """
        try:
            conn = sqlite3.connect('databases/users.db')
            cursor = conn.cursor()
            cursor.execute("SELECT * FROM users WHERE user_id = ?", (session,))
            data = cursor.fetchone()
            conn.close()

            if data:
                # Map data to a dictionary for better usability
                user_data = {
                    "id": data[0],
                    "account_creation_timestamp": data[1],
                    "user_id": data[2],
                    "username": data[4]
                }
                return user_data
            else:
                return None
        except sqlite3.Error as e:
            print(f"Database error: {e}")
            return None