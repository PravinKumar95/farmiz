import { test, expect } from '@playwright/test';

// Real user credentials loaded from .env or GitHub Secrets
const e2eEmail = process.env.E2E_EMAIL?.trim();
const e2ePassword = process.env.E2E_PASSWORD?.trim();
const hasRealCredentials = Boolean(e2eEmail && e2ePassword);

test.describe('Authentication Flow (Sign Up, Sign In, Sign Out)', () => {
  test.beforeEach(async ({ page }) => {
    // Clear localStorage before each test to ensure clean session state
    await page.goto('/farmiz/');
    await page.evaluate(() => localStorage.clear());
  });

  test('should display Sign In page by default', async ({ page }) => {
    await page.goto('/farmiz/signin');
    await expect(page.getByText('Welcome back')).toBeVisible({ timeout: 15000 });
    await expect(page.locator('#signin_email')).toBeVisible();
    await expect(page.locator('#signin_password')).toBeVisible();
    await expect(page.getByRole('button', { name: 'Sign In' })).toBeVisible();
  });

  /**
   * ONLY Signup and OTP verification flows are mocked via page.route()
   * to avoid creating un-deletable temporary accounts in Neon Auth.
   */
  test('should complete the Sign Up and OTP Verification flow (Mocked)', async ({ page }) => {
    // Match any endpoint containing /api/auth/signup using regex
    await page.route(/\/api\/auth\/signup/, async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ message: 'Verification code sent to your email.' }),
      });
    });

    // Match any endpoint containing /api/auth/verify-email using regex
    await page.route(/\/api\/auth\/verify-email/, async (route) => {
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ message: 'Email verified successfully!' }),
      });
    });

    await page.goto('/farmiz/signin');
    await expect(page.getByText('Welcome back')).toBeVisible({ timeout: 15000 });

    // Click link to navigate to Sign Up
    await page.getByRole('link', { name: 'Sign up' }).click();
    await expect(page.getByText('Create an account', { exact: true })).toBeVisible({ timeout: 10000 });

    // Fill out registration form
    await page.locator('#signup_name').fill('Test User');
    await page.locator('#signup_email').fill('testuser@example.com');
    await page.locator('#signup_password').fill('SecurePassword123!');

    // Submit Sign Up form
    await page.getByRole('button', { name: 'Sign Up' }).click();

    // Verify transition to OTP Verification step
    await expect(page.getByText('Check Your Email')).toBeVisible({ timeout: 15000 });
    await expect(page.getByText('testuser@example.com')).toBeVisible();

    // Enter 6-digit OTP code
    const otpInput = page.getByPlaceholder('Enter 6-digit code');
    await expect(otpInput).toBeVisible();
    await otpInput.fill('123456');

    // Click Verify Email button
    await page.getByRole('button', { name: 'Verify Email' }).click();

    // Verify transition to Email Verified success screen
    await expect(page.getByText('Email Verified!')).toBeVisible({ timeout: 15000 });

    // Click "Proceed to Sign In"
    await page.getByRole('link', { name: 'Proceed to Sign In' }).click();
    await expect(page.getByText('Welcome back')).toBeVisible({ timeout: 10000 });
  });

  /**
   * Real API Flow: Sign In with real user credentials from .env/secrets and Sign Out
   */
  test('should successfully Sign In and then Sign Out (Real API)', async ({ page }) => {
    test.skip(!hasRealCredentials, 'Skipped: Set E2E_EMAIL and E2E_PASSWORD secrets in GitHub Actions or .env to run Real API sign-in test');

    await page.goto('/farmiz/signin');
    await expect(page.getByText('Welcome back')).toBeVisible({ timeout: 15000 });

    // Fill real user credentials
    await page.locator('#signin_email').fill(e2eEmail!);
    await page.locator('#signin_password').fill(e2ePassword!);

    // Submit Sign In to REAL backend API
    await page.getByRole('button', { name: 'Sign In' }).click();

    // Verify transition to Authenticated Layout & Dashboard
    await expect(page.getByText('Navigation')).toBeVisible({ timeout: 15000 });
    await expect(page.getByRole('heading', { name: /Dashboard/ })).toBeVisible();

    // Verify sign-out button is present in sidebar footer
    const signOutBtn = page.getByRole('button', { name: /Sign Out/i }).first();
    await expect(signOutBtn).toBeVisible();

    // Execute Sign Out
    await signOutBtn.click();

    // Verify redirected back to Sign In screen
    await expect(page.getByText('Welcome back')).toBeVisible({ timeout: 10000 });
    await expect(page.locator('#signin_email')).toBeVisible();
  });

  /**
   * Real API Flow: Display error message on invalid credentials from real server response
   */
  test('should display error message on invalid credentials (Real API)', async ({ page }) => {
    await page.goto('/farmiz/signin');
    await expect(page.getByText('Welcome back')).toBeVisible({ timeout: 15000 });

    // Submit invalid credentials to REAL backend API
    await page.locator('#signin_email').fill('nonexistent_user_xyz@example.com');
    await page.locator('#signin_password').fill('InvalidPassword999!');

    await page.getByRole('button', { name: 'Sign In' }).click();

    // Verify error banner rendered from real backend error response
    await expect(page.locator('div.bg-red-50, div.dx-toast-description-7088ee25').first()).toBeVisible({ timeout: 15000 });
  });
});
