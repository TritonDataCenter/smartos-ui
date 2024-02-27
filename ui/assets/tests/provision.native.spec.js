/*
 *
 * This test assumes first-run.spec.js has already been executed successfully
 * or the required images have already been imported.
 *
 * This test will provision joyent and joyent-minimal instances and confirm
 * expected behavior.
 *
 */
const { test } = require('@playwright/test')
const { images } = require('./config')
const LoginPage = require('./models/LoginPage')
const ProvisionPage = require('./models/ProvisionPage')

test('Create Joyent (all defaults)', async ({ page }) => {
  const loginPage = new LoginPage(page)
  await loginPage.login()

  const provisionPage = new ProvisionPage(page)
  await provisionPage.goto()

  let image = images.find(image => image.for.indexOf('joyent') !== -1)

  await provisionPage.selectImage(image.uuid)
  await provisionPage.confirmBrand('joyent')
  await provisionPage.validate()
  await provisionPage.create()
  await provisionPage.createSuccess()
  
})