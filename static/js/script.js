// Auto-hide alerts after 5 seconds
document.addEventListener('DOMContentLoaded', function() {
    const alerts = document.querySelectorAll('.alert');
    alerts.forEach(function(alert) {
        setTimeout(function() {
            const bsAlert = new bootstrap.Alert(alert);
            bsAlert.close();
        }, 5000);
    });
});

// Focus on title input when modal opens
document.addEventListener('DOMContentLoaded', function() {
    const addTaskModal = document.getElementById('addTaskModal');
    if (addTaskModal) {
        addTaskModal.addEventListener('shown.bs.modal', function() {
            document.getElementById('title').focus();
        });
    }
});
