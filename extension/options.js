document.addEventListener('DOMContentLoaded', async function () {
    const form = document.getElementById('settingsForm');
    const serverUrlInput = document.getElementById('serverUrl');
    const saveBtn = document.getElementById('saveBtn');
    const resetBtn = document.getElementById('resetBtn');
    const testBtn = document.getElementById('testBtn');
    const statusElement = document.getElementById('status');
    const connectionStatus = document.getElementById('connectionStatus');
    const statusIndicator = document.getElementById('statusIndicator');
    const statusText = document.getElementById('statusText');

    const DEFAULT_SERVER_URL = 'http://localhost:8000';

    // Load current settings
    await loadSettings();

    // Form submit handler
    form.addEventListener('submit', async function (event) {
        event.preventDefault();

        const serverUrl = serverUrlInput.value.trim();

        if (!isValidUrl(serverUrl)) {
            showStatus('Please enter a valid URL', 'error');
            return;
        }

        // Remove trailing slash
        const cleanUrl = serverUrl.replace(/\/$/, '');

        try {
            saveBtn.disabled = true;
            saveBtn.textContent = 'Saving...';

            await browser.storage.sync.set({
                serverUrl: cleanUrl
            });

            showStatus('Settings saved successfully!', 'success');

        } catch (error) {
            console.error('Error saving settings:', error);
            showStatus('Error saving settings', 'error');
        } finally {
            saveBtn.disabled = false;
            saveBtn.textContent = 'Save Settings';
        }
    });

    // Reset button handler
    resetBtn.addEventListener('click', function () {
        serverUrlInput.value = DEFAULT_SERVER_URL;
        showStatus('Settings reset to default', 'success');
    });

    // Test connection button handler
    testBtn.addEventListener('click', async function () {
        const serverUrl = serverUrlInput.value.trim();

        if (!isValidUrl(serverUrl)) {
            showConnectionStatus('Invalid URL', false);
            return;
        }

        await testConnection(serverUrl.replace(/\/$/, ''));
    });

    async function loadSettings() {
        try {
            const result = await browser.storage.sync.get(['serverUrl']);
            serverUrlInput.value = result.serverUrl || DEFAULT_SERVER_URL;
        } catch (error) {
            console.error('Error loading settings:', error);
            serverUrlInput.value = DEFAULT_SERVER_URL;
        }
    }

    async function testConnection(serverUrl) {
        testBtn.disabled = true;
        testBtn.textContent = 'Testing...';
        showConnectionStatus('Testing connection...', null, true);

        try {
            // Test the search endpoint with a simple query
            const response = await fetch(`${serverUrl}/search_text?q=test&limit=1`, {
                method: 'GET',
                headers: {
                    'Accept': 'application/json'
                }
            });

            if (response.ok) {
                // Try to parse JSON to make sure it's a valid API response
                await response.json();
                showConnectionStatus('Connected successfully!', true);
            } else {
                showConnectionStatus(`Server error: ${response.status} ${response.statusText}`, false);
            }

        } catch (error) {
            console.error('Connection test error:', error);

            if (error.name === 'TypeError' && error.message.includes('fetch')) {
                showConnectionStatus('Cannot reach server - check URL and ensure server is running', false);
            } else {
                showConnectionStatus(`Connection failed: ${error.message}`, false);
            }
        } finally {
            testBtn.disabled = false;
            testBtn.textContent = 'Test Connection';
        }
    }

    function showConnectionStatus(message, isConnected, isTesting = false) {
        connectionStatus.style.display = 'flex';
        statusText.textContent = message;

        statusIndicator.className = 'status-indicator';
        if (isTesting) {
            statusIndicator.classList.add('testing');
        } else if (isConnected) {
            statusIndicator.classList.add('connected');
        } else {
            statusIndicator.classList.add('disconnected');
        }
    }

    function showStatus(message, type) {
        statusElement.textContent = message;
        statusElement.className = `status ${type}`;
        statusElement.style.display = 'block';

        // Auto-hide after 5 seconds
        setTimeout(() => {
            statusElement.style.display = 'none';
        }, 5000);
    }

    function isValidUrl(string) {
        try {
            const url = new URL(string);
            return url.protocol === 'http:' || url.protocol === 'https:';
        } catch (_) {
            return false;
        }
    }
});