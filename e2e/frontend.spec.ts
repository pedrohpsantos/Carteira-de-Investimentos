import { test, expect } from '@playwright/test';

test.describe('Wallet Live Frontend E2E', () => {
  test('should redirect to login if not authenticated', async ({ page }) => {
    await page.goto('/');
    await expect(page).toHaveURL(/.*\/login/);
  });

  test('should allow user to login and see dashboard', async ({ page }) => {
    await page.goto('/login');
    
    // Fill the login form
    // Note: This relies on the local DB having a user or the test creating one.
    // In our Rust logic, if the user doesn't exist, it registers them automatically!
    const testUsername = `user_${Date.now()}`;
    await page.fill('input[name="username"]', testUsername);
    await page.fill('input[name="password"]', 'password123');
    await page.click('button[type="submit"]');

    // Should redirect to dashboard
    await expect(page).toHaveURL('/');
    
    // Should see the welcome message
    await expect(page.locator('h1')).toContainText('Bem-vindo');
  });
});
