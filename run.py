import ssl
import os
from app import create_app, socketio

app = create_app()

if __name__ == '__main__':
    # SSL setup (optional)
    cert_file = os.path.join(os.getcwd(), 'app/cert.pem')
    key_file = os.path.join(os.getcwd(), 'app/key.pem')

    if not os.path.exists(cert_file) or not os.path.exists(key_file):
        print("Error: SSL certificate or key not found!")
    else:
        context = ssl.create_default_context(ssl.Purpose.CLIENT_AUTH)
        context.load_cert_chain(certfile=cert_file, keyfile=key_file)
        socketio.run(app, host='0.0.0.0', port=50100, debug=True, ssl_context=context)
