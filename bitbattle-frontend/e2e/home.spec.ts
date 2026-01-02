import { test, expect } from '@playwright/test';

test.describe('Home Page', () => {
    test('should display the BitBattle logo and title', async ({ page }) => {
        await page.goto('/');
        await expect(page.locator('h1')).toContainText('BitBattle');
        await expect(page.getByText('Competitive coding arena')).toBeVisible();
    });

    test('should show Create Room button', async ({ page }) => {
        await page.goto('/');
        await expect(page.getByRole('button', { name: 'Create Room' })).toBeVisible();
    });

    test('should show Join Room button', async ({ page }) => {
        await page.goto('/');
        await expect(page.getByRole('button', { name: 'Join Room' })).toBeVisible();
    });

    test('should show Quick Match button', async ({ page }) => {
        await page.goto('/');
        await expect(page.getByRole('button', { name: 'Quick Match' })).toBeVisible();
    });

    test('should show guest username by default', async ({ page }) => {
        await page.goto('/');
        await expect(page.getByText('Playing as')).toBeVisible();
        await expect(page.getByText(/guest_\d{4}/)).toBeVisible();
    });

    test('should show how it works section', async ({ page }) => {
        await page.goto('/');
        await expect(page.getByText('How it works')).toBeVisible();
    });
});

test.describe('Create Room Flow', () => {
    test('should navigate to create room screen', async ({ page }) => {
        await page.goto('/');
        await page.getByRole('button', { name: 'Create Room' }).click();
        await expect(page.getByText('Difficulty')).toBeVisible();
        await expect(page.getByText('Players')).toBeVisible();
    });

    test('should show difficulty options', async ({ page }) => {
        await page.goto('/');
        await page.getByRole('button', { name: 'Create Room' }).click();
        await expect(page.getByRole('button', { name: 'Random' })).toBeVisible();
        await expect(page.getByRole('button', { name: 'Easy' })).toBeVisible();
        await expect(page.getByRole('button', { name: 'Medium' })).toBeVisible();
        await expect(page.getByRole('button', { name: 'Hard' })).toBeVisible();
    });

    test('should show game mode options', async ({ page }) => {
        await page.goto('/');
        await page.getByRole('button', { name: 'Create Room' }).click();
        await expect(page.getByRole('button', { name: 'Casual' })).toBeVisible();
        await expect(page.getByRole('button', { name: 'Ranked' })).toBeVisible();
    });

    test('should allow selecting difficulty', async ({ page }) => {
        await page.goto('/');
        await page.getByRole('button', { name: 'Create Room' }).click();
        await page.getByRole('button', { name: 'Easy' }).click();
        // The Easy button should be highlighted (has green background)
        await expect(page.getByRole('button', { name: 'Easy' })).toHaveClass(/bg-green/);
    });

    test('should return to home when Cancel is clicked', async ({ page }) => {
        await page.goto('/');
        await page.getByRole('button', { name: 'Create Room' }).click();
        await page.getByRole('button', { name: 'Cancel' }).click();
        await expect(page.getByRole('button', { name: 'Quick Match' })).toBeVisible();
    });
});

test.describe('Join Room Flow', () => {
    test('should navigate to join room screen', async ({ page }) => {
        await page.goto('/');
        await page.getByRole('button', { name: 'Join Room' }).click();
        await expect(page.getByText('Room Code')).toBeVisible();
        await expect(page.getByPlaceholder('SWIFT-CODER-1234')).toBeVisible();
    });

    test('should convert room code to uppercase', async ({ page }) => {
        await page.goto('/');
        await page.getByRole('button', { name: 'Join Room' }).click();
        await page.getByPlaceholder('SWIFT-CODER-1234').fill('swift-coder-1234');
        await expect(page.getByPlaceholder('SWIFT-CODER-1234')).toHaveValue('SWIFT-CODER-1234');
    });

    test('should disable Join button when room code is empty', async ({ page }) => {
        await page.goto('/');
        await page.getByRole('button', { name: 'Join Room' }).click();
        const joinButtons = await page.getByRole('button', { name: 'Join Room' }).all();
        const joinButton = joinButtons[joinButtons.length - 1];
        await expect(joinButton).toBeDisabled();
    });

    test('should enable Join button when room code is entered', async ({ page }) => {
        await page.goto('/');
        await page.getByRole('button', { name: 'Join Room' }).click();
        await page.getByPlaceholder('SWIFT-CODER-1234').fill('TEST-CODE-1234');
        const joinButtons = await page.getByRole('button', { name: 'Join Room' }).all();
        const joinButton = joinButtons[joinButtons.length - 1];
        await expect(joinButton).toBeEnabled();
    });

    test('should return to home when Cancel is clicked', async ({ page }) => {
        await page.goto('/');
        await page.getByRole('button', { name: 'Join Room' }).click();
        await page.getByRole('button', { name: 'Cancel' }).click();
        await expect(page.getByRole('button', { name: 'Quick Match' })).toBeVisible();
    });
});

test.describe('Navigation', () => {
    test('should have navigation links', async ({ page }) => {
        await page.goto('/');
        await expect(page.getByRole('link', { name: 'Play' })).toBeVisible();
        await expect(page.getByRole('link', { name: 'Leaderboard' })).toBeVisible();
    });

    test('should navigate to leaderboard', async ({ page }) => {
        await page.goto('/');
        await page.getByRole('link', { name: 'Leaderboard' }).click();
        await expect(page).toHaveURL('/leaderboard');
    });
});
