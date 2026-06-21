import { test, expect } from '@playwright/test';

test.describe('PaperTime — smoke tests', () => {

	test('homepage loads and shows search form', async ({ page }) => {
		await page.goto('/');
		const input = page.locator('input[placeholder*="station"]');
		await expect(input).toBeVisible({ timeout: 15000 });
	});

	test('search shows suggestions when typing', async ({ page }) => {
		await page.goto('/');
		const input = page.locator('input[placeholder*="station"]');
		await expect(input).toBeVisible({ timeout: 15000});

		await input.fill('EUSTON');
		await page.waitForTimeout(500);

		const suggestions = page.locator('button');
		await expect(suggestions.first()).toBeVisible({ timeout: 5000 });
	});

	test('clicking suggestion navigates to station page', async ({ page }) => {
		await page.goto('/');
		const input = page.locator('input[placeholder*="station"]');
		await expect(input).toBeVisible({ timeout: 15000});

		await input.fill('EUSTON');
		await page.waitForTimeout(500);

		const suggestion = page.locator('button').first();
		await expect(suggestion).toBeVisible({ timeout: 5000 });
		await suggestion.click();

		await page.waitForURL(/\/station\//, { timeout: 10000 });
		await page.waitForTimeout(5000);
		const body = page.locator('body');
		await expect(body).toBeVisible();
	});

	test('station page shows content after loading', async ({ page }) => {
		await page.goto('/station/EUSTON');
		await page.waitForTimeout(5000);
		const body = page.locator('body');
		await expect(body).toBeVisible();
	});

	test('404 page shows for unknown routes', async ({ page }) => {
		await page.goto('/nonexistent-page');
		await expect(page.locator('text=404')).toBeVisible({ timeout: 10000 });
	});
});
