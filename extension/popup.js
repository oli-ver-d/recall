document.addEventListener('DOMContentLoaded', async function () {
    const currentUrlElement = document.getElementById('currentUrl');
    const tagsInput = document.getElementById('tagsInput');
    const saveBtn = document.getElementById('saveBtn');
    const saveText = document.getElementById('saveText');
    const optionsBtn = document.getElementById('optionsBtn');
    const statusElement = document.getElementById('status');

    let currentTab = null;

    // Get current tab info
    try {
        const tabs = await browser.tabs.query({ active: true, currentWindow: true });
        currentTab = tabs[0];
        currentUrlElement.textContent = currentTab.url;
    } catch (error) {
        currentUrlElement.textContent = 'Error loading URL';
        console.error('Error getting current tab:', error);
    }

    // Load saved tags for this URL (if any)
    try {
        const result = await browser.storage.local.get([`tags_${currentTab.url}`]);
        if (result[`tags_${currentTab.url}`]) {
            tagsInput.value = result[`tags_${currentTab.url}`];
        }
    } catch (error) {
        console.error('Error loading saved tags:', error);
    }

    // Save button click handler
    saveBtn.addEventListener('click', async function () {
        if (!currentTab) {
            showStatus('Error: No active tab found', 'error');
            return;
        }

        const tags = tagsInput.value.trim();
        const tagArray = tags ? tags.split(',').map(tag => tag.trim()).filter(tag => tag) : [];

        // Show loading state
        saveBtn.disabled = true;
        saveText.innerHTML = '<span class="spinner"></span>Saving...';
        showStatus('Saving page to Recall...', 'loading');

        try {
            // Get server URL from settings
            const settings = await browser.storage.sync.get(['serverUrl']);
            const serverUrl = settings.serverUrl || 'http://localhost:8000';

            // Make API request
            const response = await fetch(`${serverUrl}/save/`, {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    url: currentTab.url,
                    tags: tagArray
                })
            });

            if (response.ok) {
                const result = await response.json();

                // Save tags for this URL
                if (tags) {
                    await browser.storage.local.set({
                        [`tags_${currentTab.url}`]: tags
                    });
                }

                showStatus(`✓ Saved successfully (ID: ${result.id})`, 'success');

                // // Close popup after 2 seconds
                // setTimeout(() => {
                //     window.close();
                // }, 2000);

            } else {
                const errorText = await response.text();
                showStatus(`Error: ${errorText}`, 'error');
            }

        } catch (error) {
            console.error('Save error:', error);
            showStatus(`Error: ${error.message}`, 'error');
        } finally {
            // Reset button state
            saveBtn.disabled = false;
            saveText.textContent = 'Save Page';
        }
    });

    // Options button click handler
    optionsBtn.addEventListener('click', function () {
        browser.runtime.openOptionsPage();
    });

    // Enter key in tags input
    tagsInput.addEventListener('keypress', function (event) {
        if (event.key === 'Enter') {
            saveBtn.click();
        }
    });

    // Focus tags input
    tagsInput.focus();

    function showStatus(message, type) {
        statusElement.textContent = message;
        statusElement.className = `status ${type}`;
        statusElement.style.display = 'block';

        // Auto-hide success messages
        if (type === 'success') {
            setTimeout(() => {
                statusElement.style.display = 'none';
            }, 3000);
        }
    }
});