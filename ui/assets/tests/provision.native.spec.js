/*
 *
 * This test assumes first-run.spec.js has already been executed successfully
 * or the required images have already been imported.
 *
 * This test will provision and then delete joyent and joyent-minimal instances
 * and confirm expected behavior.
 *
 */
const { test } = require('@playwright/test')
const config = require('./config')
const LoginPage = require('./models/LoginPage')
const ProvisionPage = require('./models/ProvisionPage')
const InstancesPage = require('./models/InstancesPage')
const InstanceJoyentPage = require('./models/InstanceJoyentPage')
const InstanceJoyentMinimalPage = require('./models/InstanceJoyentMinimalPage')

// For additional manual testing, set to false so instances can be inspected
const deleteInstances = true

const ips = Array.from(config.nic.ips.native)

test('Create Joyent (all defaults)', async ({ page }, {title}) => {
  test.setTimeout(config.instanceCreateTimeout.timeout)
  const brand = 'joyent'
  const loginPage = new LoginPage(page)
  await loginPage.login()

  const provisionPage = new ProvisionPage(page)
  await provisionPage.goto()

  let image = config.images.find(image => image.for.indexOf(brand) !== -1)

  await provisionPage.setAlias(title)
  await provisionPage.selectImage(image.uuid)
  await provisionPage.isBrand(brand)
  await provisionPage.validate()
  await provisionPage.create()
  await provisionPage.createSuccess()
  const uuid = await provisionPage.viewDetails()

  const instancePage = new InstanceJoyentPage(page, uuid)
  await instancePage.isExpectedBrand()
  await instancePage.isRunning()
  await instancePage.stop()
  await instancePage.start()
  if (deleteInstances) {
    await instancePage.deleteInstance()
  }
})

test('Create Joyent Minimal (all defaults)', async ({ page }, {title}) => {
  test.setTimeout(config.instanceCreateTimeout.timeout)
  const brand = 'joyent-minimal'
  const loginPage = new LoginPage(page)
  await loginPage.login()

  const provisionPage = new ProvisionPage(page)
  await provisionPage.goto()

  let image = config.images.find(image => image.for.indexOf(brand) !== -1)

  await provisionPage.setAlias(title)
  await provisionPage.selectImage(image.uuid)
  await provisionPage.selectBrand(brand)
  await provisionPage.isBrand(brand)
  await provisionPage.validate()
  await provisionPage.create()
  await provisionPage.createSuccess()
  const uuid = await provisionPage.viewDetails()

  const instancePage = new InstanceJoyentMinimalPage(page, uuid)
  await instancePage.isExpectedBrand()
  await instancePage.isRunning()
  await instancePage.stop()
  await instancePage.start()
  if (deleteInstances) {
    await instancePage.deleteInstance()
  }
})

test('Joyent (512MiB RAM, 8GiB Quota, Ipv4 Static)', async ({ page }, {title}) => {
  test.setTimeout(config.instanceCreateTimeout.timeout)
  const brand = 'joyent'
  const loginPage = new LoginPage(page)
  await loginPage.login()

  const ip = ips.pop()

  const provisionPage = new ProvisionPage(page)
  await provisionPage.goto()

  let image = config.images.find(image => image.for.indexOf(brand) !== -1)

  await provisionPage.setAlias(title)
  await provisionPage.selectImage(image.uuid)
  await provisionPage.isBrand(brand)
  await provisionPage.setRam(512)
  await provisionPage.setQuota(8)
  await provisionPage.setSSHKey(config.sshKey)
  await provisionPage.selectNicTag(config.nic.tag)
  await provisionPage.setResolvers(config.nic.resolvers)
  await provisionPage.selectIPv4Setup('static')
  await provisionPage.setIPv4Address(ip)
  await provisionPage.selectIpv4Prefix(config.nic.ipv4Prefix)
  await provisionPage.setIpv4Gateway(config.nic.gateway)
  await provisionPage.validate()
  await provisionPage.create()
  await provisionPage.createSuccess()
  const uuid = await provisionPage.viewDetails()

  const instancePage = new InstanceJoyentPage(page, uuid)
  await instancePage.isExpectedBrand()
  await instancePage.isRunning()
  await instancePage.stop()
  await instancePage.start()
  if (deleteInstances) {
    await instancePage.deleteInstance()
  }
})

test('Joyent (1024MiB RAM, 16GiB Quota, Delegate Dataset, Ipv4 Static)', async ({ page }, {title}) => {
  test.setTimeout(config.instanceCreateTimeout.timeout)
  const brand = 'joyent'
  const loginPage = new LoginPage(page)
  await loginPage.login()

  const ip = ips.pop()

  const provisionPage = new ProvisionPage(page)
  await provisionPage.goto()

  let image = config.images.find(image => image.for.indexOf(brand) !== -1)

  await provisionPage.setAlias(title)
  await provisionPage.selectImage(image.uuid)
  await provisionPage.isBrand(brand)
  await provisionPage.setRam(1024)
  await provisionPage.setQuota(16)
  await provisionPage.setDelegateDataset()
  await provisionPage.setSSHKey(config.sshKey)
  await provisionPage.selectNicTag(config.nic.tag)
  await provisionPage.setResolvers(config.nic.resolvers)
  await provisionPage.selectIPv4Setup('static')
  await provisionPage.setIPv4Address(ip)
  await provisionPage.selectIpv4Prefix(config.nic.ipv4Prefix)
  await provisionPage.setIpv4Gateway(config.nic.gateway)
  await provisionPage.validate()
  await provisionPage.create()
  await provisionPage.createSuccess()
  const uuid = await provisionPage.viewDetails()

  const instancePage = new InstanceJoyentPage(page, uuid)
  await instancePage.isExpectedBrand()
  await instancePage.isRunning()
  await instancePage.stop()
  await instancePage.start()
  if (deleteInstances) {
    await instancePage.deleteInstance()
  }
})
