const { invoke } = window.__TAURI__.tauri;

// Function to get running applications
function getOpenWindowApps() {
  invoke('get_open_window_apps').then((apps) => {
    const appList = document.getElementById('app-list');
    appList.innerHTML = '';  // Clear previous list

    apps.forEach((app) => {
      const button = document.createElement('button');
      button.textContent = `${app.application_name} - ${app.title}`;
      button.onclick = () => activateWindow(app.hwnd);  // Add click event to activate the window
      appList.appendChild(button);
    });
    
  }).catch((error) => {
    console.error('Error:', error);
  });
}

// Function to activate a window
function activateWindow(hwnd) {
  invoke('activate_window', { hwnd }).then((success) => {
    if (!success) {
      console.error('Failed to activate window');
    }
  }).catch((error) => {
    console.error('Error activating window:', error);
  });
}

// Call the function on page load or button click
window.onload = function() {
  getOpenWindowApps();
};