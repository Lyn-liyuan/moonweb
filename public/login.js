
function showLogin(closeable=false) {
    const options = {
        placement: 'bottom-right',
        backdrop: 'dynamic',
        backdropClasses:
            'bg-gray-900/50 dark:bg-gray-900/80 fixed inset-0 z-40',
        closable: closeable,
    };
    
    const instanceOptions = {
        id: 'login-modal',
        override: true
    };
    const $targetEl = document.getElementById('login-modal');
    const modal = new Modal($targetEl, options, instanceOptions);
    modal.show();
}

function closeLogin() {
    const options = {
        placement: 'bottom-right',
        backdrop: 'dynamic',
        backdropClasses:
            'bg-gray-900/50 dark:bg-gray-900/80 fixed inset-0 z-40',
        closable: true,
    };
    
    const instanceOptions = {
        id: 'login-modal',
        override: true
    };
    const $targetEl = document.getElementById('login-modal');
    const modal = new Modal($targetEl, options, instanceOptions);
    modal.hide();
}
