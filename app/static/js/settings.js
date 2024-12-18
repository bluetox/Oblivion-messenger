function loadParams() {
    const settings = JSON.parse(localStorage.getItem('userSettings'));
    const accountInfoElement = document.getElementById('accountInfo');
    const authButtonsElement = document.getElementById('authButtons');

    if (settings && settings.isloggedin === true && settings.username) {
      accountInfoElement.innerHTML = `Hello, ${settings.username}!`;
      authButtonsElement.style.display = 'none';
    } else {
      accountInfoElement.innerHTML = 'You are not logged in.';
      authButtonsElement.style.display = 'block';
    }
  }

  function showSection(sectionId) {
    const sections = document.querySelectorAll('.section');
    sections.forEach(section => section.classList.remove('active'));

    document.getElementById(sectionId).classList.add('active');

    const menuItems = document.querySelectorAll('.menu-item');
    menuItems.forEach(item => item.classList.remove('active'));
    event.target.classList.add('active');
  }

  function clearDb(name) {
    let request = indexedDB.deleteDatabase(name);

    request.onsuccess = function() {
      console.log(`IndexedDB ${name} cleared successfully.`);
    };

    request.onerror = function(event) {
      console.error('Error clearing IndexedDB:', event.target.error);
    };

  }

  function disappear() {

    clearDb("Messages");
    clearDb("chatIndex");
    localStorage.clear();
    console.log('LocalStorage cleared.');
    document.cookie.split(';').forEach(function(cookie) {
      const cookieName = cookie.split('=')[0];
      document.cookie = cookieName + '=; expires=Thu, 01 Jan 1970 00:00:00 GMT; path=/';
    });
    if ('caches' in window) {
      caches.keys().then(function(cacheNames) {
        cacheNames.forEach(function(cacheName) {
          caches.delete(cacheName);
        });
        console.log('All caches cleared.');
      });
    }
  }
  loadParams();