<div class="empty-state">
    <div class="icon-wrapper glass">
        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="2" stroke="currentColor" class="icon">
            <path stroke-linecap="round" stroke-linejoin="round" d="M21 21l-5.197-5.197m0 0A7.5 7.5 0 105.196 5.196a7.5 7.5 0 0010.607 10.607z" />
        </svg>
    </div>
    <h3 class="title text-gradient">No results found</h3>
    <p class="description">We couldn't find anything matching your search. Try adjusting your filters or try a broader term.</p>
</div>

<style>
    .empty-state {
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        padding: var(--spacing-2xl) var(--spacing-lg);
        text-align: center;
        color: var(--color-text-secondary);
    }

    .icon-wrapper {
        width: 80px;
        height: 80px;
        border-radius: var(--radius-xl);
        display: flex;
        align-items: center;
        justify-content: center;
        margin-bottom: var(--spacing-xl);
        border: 1px solid var(--color-border-bright);
        box-shadow: var(--shadow-md);
    }

    .icon {
        width: 32px;
        height: 32px;
        color: var(--color-accent);
        opacity: 0.8;
    }

    .title {
        font-size: var(--text-2xl);
        font-weight: 800;
        margin-bottom: var(--spacing-sm);
        letter-spacing: -0.03em;
    }

    .description {
        font-size: var(--text-base);
        max-width: 360px;
        line-height: 1.6;
        opacity: 0.8;
    }
</style>
