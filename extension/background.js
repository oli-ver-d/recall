// Background script for Recall Saver extension

// Set default settings on install
browser.runtime.onInstalled.addListener(async function (details) {
    if (details.reason === 'install') {
        try {
            // Set default server URL
            await browser.storage.sync.set({
                serverUrl: 'http://localhost:8000'
            });

            console.log('Recall Saver extension installed with default settings');
        } catch (error) {
            console.error('Error setting default settings:', error);
        }
    }
});

// Handle context menu clicks (if we add context menus in the future)
browser.contextMenus.onClicked.addListener(function (info, tab) {
    // Future: Add context menu integration
});

// Optional: Add keyboard shortcut handler
browser.commands.onCommand.addListener(function (command) {
    if (command === 'save-page') {
        // Open the popup programmatically
        browser.browserAction.openPopup();
    }
});

// Optional: Handle messages from content scripts or popup
browser.runtime.onMessage.addListener(function (request, sender, sendResponse) {
    // Handle any background processing if needed
    return false; // Not handling async responses for now
});