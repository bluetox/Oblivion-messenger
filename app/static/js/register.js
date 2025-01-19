let url =  window.location.origin;
    function setCookie(name, value, days) {
      const date = new Date();
      date.setTime(date.getTime() + (days * 24 * 60 * 60 * 1000));
      const expires = "expires=" + date.toUTCString();
      document.cookie = `${name}=${value}; ${expires}; path=/; Secure`;
    }


    document.getElementById('createAccountForm').addEventListener('submit', async function(event) {
      event.preventDefault();

      const username = document.getElementById('username').value;
      const password = document.getElementById('password').value;
      const messageDiv = document.getElementById('message');

      const hashedPassword = await hashPassword(password);
      
      fetch('/api/create_account', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ username: username, password: hashedPassword })
      })
      .then(response => response.json())
      .then(data => {
        if (data.status === 'registered') {
          setCookie('session_id', data.token, 7);
          localStorage.setItem('userSettings', JSON.stringify({ isloggedin : true , username : username}));
          messageDiv.textContent = 'Account created successfully!';
          window.open(`${url}/chat`, '_blank');
        } else {
          messageDiv.textContent = 'Error: ' + (data.error || 'Unknown error');
        }
      })
      .catch(error => {
        messageDiv.textContent = 'There was an error processing your request';
        console.error('Error:', error);
      });
    });

    async function hashPassword(password) {
      const encoder = new TextEncoder();
      const data = encoder.encode(password);
      const hashBuffer = await crypto.subtle.digest('SHA-256', data);
      const hashArray = Array.from(new Uint8Array(hashBuffer));
      const hashHex = hashArray.map(byte => byte.toString(16).padStart(2, '0')).join('');
      return hashHex;
    }