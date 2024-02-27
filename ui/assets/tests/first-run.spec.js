/*
 *
 * This test assumes a fresh SmartOS instance with no images or instances and
 * a fresh cache.
 *
 * It will import the images necessary for other test suites.
 *
 * When testing a SmartOS instance that already has images or instances you may
 * skip this suite, but should ensure the necessary images are installed.
 *
 */
const { test } = require('@playwright/test')
const { images, imageImportTimeout } = require('./config')
const LoginPage = require('./models/LoginPage')
const InstancesPage = require('./models/InstancesPage')
const ImagesPage = require('./models/ImagesPage')
const DashboardPage = require('./models/DashboardPage')
const AvailableImagesPage = require('./models/AvailableImagesPage')

test.describe.configure({ mode: 'serial' })

test('Login Failure', async ({ page }) => {
  const loginPage = new LoginPage(page)
  await loginPage.goto()
  await loginPage.loginFailure()
})

test('Login Success', async ({ page }) => {
  const loginPage = new LoginPage(page)
  await loginPage.goto()
  await loginPage.loginSuccess()
  await loginPage.firstLoad()

  const instancePage = new InstancesPage(page)
  await instancePage.goto()
  await instancePage.firstLoad()

  const imagesPage = new ImagesPage(page)
  await imagesPage.goto()
  await imagesPage.firstLoad()

  const dashboardPage = new DashboardPage(page)
  await dashboardPage.goto()
  await dashboardPage.logout()
})

test('Import Images', async ({ page }) => {
  test.setTimeout(imageImportTimeout.timeout * images.length)
  const loginPage = new LoginPage(page)
  await loginPage.login()

  const imagesPage = new ImagesPage(page)
  await imagesPage.goto()
  await imagesPage.availableImages()

  const availPage = new AvailableImagesPage(page)
  await availPage.goto()

  for (const i in images) {
    await availPage.import(images[i].name, images[i].uuid)
  }
})

test('Dashboard, no instances', async ({ page }) => {
  const loginPage = new LoginPage(page)
  await loginPage.goto()
  await loginPage.loginSuccess()

  const dashboardPage = new DashboardPage(page)
  await dashboardPage.goto()
  await dashboardPage.firstLoadNoInstances()
})

test('Instances, no instances', async ({ page }) => {
  const loginPage = new LoginPage(page)
  await loginPage.goto()
  await loginPage.loginSuccess()

  const instancePage = new InstancesPage(page)
  await instancePage.goto()
  await instancePage.firstLoadNoInstances()
})
