import { expect, test } from "@playwright/test";

test("首页渲染并可交互", async ({ page }) => {
  await page.goto("/");

  await expect(page).toHaveTitle("Leptos Template");
  await expect(page.locator("h1")).toHaveText("Welcome to Leptos Template!");
  await expect(page.getByRole("button")).toHaveText("Click Me: 0");

  await page.getByRole("button").click();

  await expect(page.getByRole("button")).toHaveText("Click Me: 1");
});
