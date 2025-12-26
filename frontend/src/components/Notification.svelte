<!-- frontend/src/components/Notification.svelte -->
<script lang="ts">
    let { message, type } = $props();
    let isVisible = $state(true);

    $effect(() => {
        isVisible = true; // Reset visibility on new message
        const timer = setTimeout(() => {
            isVisible = false;
        }, 3000);

        return () => clearTimeout(timer); // Cleanup on component destruction or prop change
    });
</script>

{#if isVisible}
    <div class="notification {type}">
        {message}
    </div>
{/if}

<style>
    .notification {
        position: fixed;
        top: 20px;
        right: 20px;
        padding: 10px 20px;
        border-radius: 5px;
        box-shadow: 0 2px 5px rgba(0, 0, 0, 0.2);
        z-index: 1000;
        opacity: 1;
        transition: opacity 0.3s ease-in-out;
    }

    .notification.success {
        background-color: #4CAF50; /* Green */
        color: white;
    }

    .notification.error {
        background-color: #F44336; /* Red */
        color: white;
    }

    /* Optional: Animation for fade-out */
    .notification:not([isVisible]) {
        opacity: 0;
    }
</style>