import sqlite3
import bcrypt
from datetime import datetime

class Database:
    
    
    def add_user(self, username, user_id, password):
        try:
            conn = sqlite3.connect('databases/users.db')
            salt = bcrypt.gensalt(12)
            password = bcrypt.hashpw(password.encode('utf-8'), salt)
            cursor = conn.cursor()
            now = datetime.now().isoformat()
            cursor.execute("""
                INSERT INTO users (account_creation_timestamp, user_id, password, username)
                VALUES (?, ?, ?, ?)
            """, (now, user_id, password, username))
            conn.commit()
            conn.close()
            return True
        except sqlite3.Error as e:
            print(f"Database error: {e}")
            return False


    def check_user_presence(self, username):
        try:
            conn = sqlite3.connect('databases/users.db')
            cursor = conn.cursor()
            cursor.execute("SELECT * FROM users WHERE username = ?", (username,))
            if cursor.fetchone():
                return True
            return False
        except sqlite3.Error as e:
            print(f"Database error: {e}")
            return False
    

    def login(self, username, password):
        try:
            conn = sqlite3.connect('databases/users.db')
            cursor = conn.cursor()
            cursor.execute("SELECT * FROM users WHERE username = ?", (username,))
            data = cursor.fetchone()
            if data:
                hashed_password = data[0]
                if bcrypt.checkpw(password.encode('utf-8'), hashed_password) == True:
                    return True
            return False
        except sqlite3.Error as e:
            print(f"Database error: {e}")
            return False
