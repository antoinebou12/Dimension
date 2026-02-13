import { defineConfig, devices } from '@playwright/test';
import path from 'path';

export default defineConfig({
  testDir: './tests',
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1,
  reporter: 'html',
  use: {
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
  },
  projects: [
    {
      name: 'render',
      use: {
        ...devices['Desktop Chrome'],
        baseURL: 'http://localhost:3000',
        launchOptions: {
          args: ['--enable-unsafe-webgpu', '--ignore-gpu-blocklist'],
        },
      },
      webServer: {
        command: 'npx serve ../render/demo -l 3000',
        url: 'http://localhost:3000',
        cwd: path.dirname(__filename ?? __dirname),
        reuseExistingServer: !process.env.CI,
        timeout: 120000,
      },
    },
    {
      name: 'mathlib',
      use: {
        ...devices['Desktop Chrome'],
        baseURL: 'http://localhost:3001',
      },
      webServer: {
        command: 'npx serve ../mathlib/demo',
        url: 'http://localhost:3001',
        reuseExistingServer: !process.env.CI,
        timeout: 120000,
      },
    },
    {
      name: 'kinematics',
      use: {
        ...devices['Desktop Chrome'],
        baseURL: 'http://localhost:3002',
      },
      webServer: {
        command: 'npx serve ../kinematics/demo -p 3002',
        url: 'http://localhost:3002',
        reuseExistingServer: !process.env.CI,
        timeout: 120000,
      },
    },
  ],
});
