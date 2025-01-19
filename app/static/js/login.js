
let url =  window.location.origin;
async function hashPassword(password) {
  const encoder = new TextEncoder();
  const data = encoder.encode(password);
  const hashBuffer = await crypto.subtle.digest('SHA-256', data);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  const hashHex = hashArray.map(byte => byte.toString(16).padStart(2, '0')).join('');
  return hashHex;
}
function setCookie(name, value, days) {
  const date = new Date();
  date.setTime(date.getTime() + (days * 24 * 60 * 60 * 1000));
  const expires = "expires=" + date.toUTCString();
  document.cookie = `${name}=${value}; ${expires}; path=/; Secure`;
}
function openChat() {
  window.open(`/chat`, '_blank');
}
document.getElementById('loginForm').addEventListener('submit', async function(event) {
  event.preventDefault();
  const username = document.getElementById('username').value;
  const password = document.getElementById('password').value;
  const messageDiv = document.getElementById('message');
  const hashedPassword = await hashPassword(password);
  fetch('/api/login', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({ username, password: hashedPassword })
  })
  .then(response => response.json())
  .then(data => {
    if (data.status === 'logged_in') {
      setCookie('session_id', data.token, 7);
      localStorage.setItem('userSettings', JSON.stringify({ isloggedin : true , username : username}));
      messageDiv.textContent = 'Login successful!';
      messageDiv.style.color = '#00e676';
      openChat();
    } else {
      messageDiv.textContent = 'Error: ' + (data.error || 'Invalid username or password');
      messageDiv.style.color = '#ff4081';
    }
  })
  .catch(error => {
    messageDiv.textContent = 'There was an error processing your request';
    messageDiv.style.color = '#ff4081';
    console.error('Error:', error);
  });
});