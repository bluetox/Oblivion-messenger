import ssl
import os
from app import create_app, socketio

print("\033[38;2;255;165;0m",r"""                                                                                 
                                                                                 
         __________ ______________ ________          _     ___       ___         
         `MMMMMMMMM `MM'MMMMMMMMMM `MMMMMMMb.       dM.    `MMb     dMM'         
          MM      \  MM /   MM   \  MM    `Mb      ,MMb     MMM.   ,PMM          
  ____    MM         MM     MM      MM     MM      d'YM.    M`Mb   d'MM   ____   
 6MMMMb   MM    ,    MM     MM      MM     MM     ,P `Mb    M YM. ,P MM  6MMMMb  
MM'  `Mb  MMMMMMM    MM     MM      MM    .M9     d'  YM.   M `Mb d' MM MM'  `Mb 
     ,MM  MM    `    MM     MM      MMMMMMM9'    ,P   `Mb   M  YM.P  MM      ,MM 
    ,MM'  MM         MM     MM      MM  \M\      d'    YM.  M  `Mb'  MM     ,MM' 
  ,M'     MM         MM     MM      MM   \M\    ,MMMMMMMMb  M   YP   MM   ,M'    
,M'       MM      /  MM     MM      MM    \M\   d'      YM. M   `'   MM ,M'      
MMMMMMMM _MMMMMMMMM _MM_   _MM_    _MM_    \M\_dM_     _dMM_M_      _MM_MMMMMMMM 
                                                                                 
                                                                                 
                                                                                 ""","\033[0m")

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
