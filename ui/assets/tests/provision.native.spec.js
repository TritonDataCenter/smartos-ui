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
const InstanceJoyentPage = require('./models/InstanceJoyentPage')
const InstanceJoyentMinimalPage = require('./models/InstanceJoyentMinimalPage')
const InstanceLxPage = require('./models/InstanceLxPage')

// For additional manual testing, set to false so instances can be inspected
const deleteInstances = true

test.describe.configure({ mode: 'serial' })

test('Create Joyent (all defaults)', async ({ page }, { title }) => {
  test.setTimeout(config.instanceCreateTimeout.timeout)
  const brand = 'joyent'
  const loginPage = new LoginPage(page)
  await loginPage.login()

  const provisionPage = new ProvisionPage(page)
  await provisionPage.goto()

  const image = config.images.find(image => image.for.indexOf(brand) !== -1)

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

test('Create Joyent Minimal (all defaults)', async ({ page }, { title }) => {
  test.setTimeout(config.instanceCreateTimeout.timeout)
  const brand = 'joyent-minimal'
  const loginPage = new LoginPage(page)
  await loginPage.login()

  const provisionPage = new ProvisionPage(page)
  await provisionPage.goto()

  const image = config.images.find(image => image.for.indexOf(brand) !== -1)

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

test('Create LX (all defaults)', async ({ page }, { title }) => {
  test.setTimeout(config.instanceCreateTimeout.timeout)
  const brand = 'lx'
  const loginPage = new LoginPage(page)
  await loginPage.login()

  const provisionPage = new ProvisionPage(page)
  await provisionPage.goto()

  const image = config.images.find(image => image.for.indexOf(brand) !== -1)

  await provisionPage.setAlias(title)
  await provisionPage.selectImage(image.uuid)
  await provisionPage.selectBrand(brand)
  await provisionPage.isBrand(brand)
  await provisionPage.validate()
  await provisionPage.create()
  await provisionPage.createSuccess()
  const uuid = await provisionPage.viewDetails()

  const instancePage = new InstanceLxPage(page, uuid)
  await instancePage.isExpectedBrand()
  await instancePage.isRunning()
  await instancePage.stop()
  await instancePage.start()
  if (deleteInstances) {
    await instancePage.deleteInstance()
  }
})

test('Joyent (512MiB RAM, 8GiB Quota, Ipv4 Static)', async ({ page }, { title }) => {
  test.setTimeout(config.instanceCreateTimeout.timeout)
  const brand = 'joyent'
  const loginPage = new LoginPage(page)
  await loginPage.login()

  const ip = config.nic.ips.native[0]

  const provisionPage = new ProvisionPage(page)
  await provisionPage.goto()

  const image = config.images.find(image => image.for.indexOf(brand) !== -1)

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

test('LX (512MiB RAM, 8GiB Quota, Ipv4 Static)', async ({ page }, { title }) => {
  test.setTimeout(config.instanceCreateTimeout.timeout)
  const brand = 'lx'
  const loginPage = new LoginPage(page)
  await loginPage.login()

  const ip = config.nic.ips.native[1]

  const provisionPage = new ProvisionPage(page)
  await provisionPage.goto()

  const image = config.images.find(image => image.for.indexOf(brand) !== -1)

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

  const instancePage = new InstanceLxPage(page, uuid)
  await instancePage.isExpectedBrand()
  await instancePage.isRunning()
  await instancePage.stop()
  await instancePage.start()
  if (deleteInstances) {
    await instancePage.deleteInstance()
  }
})

test('Joyent (1024MiB RAM, 16GiB Quota, Delegate Dataset, Ipv4 Static)', async ({ page }, { title }) => {
  test.setTimeout(config.instanceCreateTimeout.timeout)
  const brand = 'joyent'
  const loginPage = new LoginPage(page)
  await loginPage.login()

  const ip = config.nic.ips.native[2]

  const provisionPage = new ProvisionPage(page)
  await provisionPage.goto()

  const image = config.images.find(image => image.for.indexOf(brand) !== -1)

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

test('LX (1024MiB RAM, 16GiB Quota, Delegate Dataset, Ipv4 Static)', async ({ page }, { title }) => {
  test.setTimeout(config.instanceCreateTimeout.timeout)
  const brand = 'lx'
  const loginPage = new LoginPage(page)
  await loginPage.login()

  const ip = config.nic.ips.native[2]

  const provisionPage = new ProvisionPage(page)
  await provisionPage.goto()

  const image = config.images.find(image => image.for.indexOf(brand) !== -1)

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

  const instancePage = new InstanceLxPage(page, uuid)
  await instancePage.isExpectedBrand()
  await instancePage.isRunning()
  await instancePage.stop()
  await instancePage.start()
  if (deleteInstances) {
    await instancePage.deleteInstance()
  }
})

test('Joyent Minimal (512MiB RAM, 8GiB Quota, Ipv4 Static)', async ({ page }, { title }) => {
  test.setTimeout(config.instanceCreateTimeout.timeout)
  const brand = 'joyent-minimal'
  const loginPage = new LoginPage(page)
  await loginPage.login()

  const ip = config.nic.ips.native[3]

  const provisionPage = new ProvisionPage(page)
  await provisionPage.goto()

  const image = config.images.find(image => image.for.indexOf(brand) !== -1)

  await provisionPage.setAlias(title)
  await provisionPage.selectImage(image.uuid)
  await provisionPage.selectBrand(brand)
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

  const instancePage = new InstanceJoyentMinimalPage(page, uuid)
  await instancePage.isExpectedBrand()
  await instancePage.isRunning()
  await instancePage.stop()
  await instancePage.start()
  if (deleteInstances) {
    await instancePage.deleteInstance()
  }
})

test('Joyent Minimal (1024MiB RAM, 16GiB Quota, Delegate Dataset, Ipv4 Static)', async ({ page }, { title }) => {
  test.setTimeout(config.instanceCreateTimeout.timeout)
  const brand = 'joyent-minimal'
  const loginPage = new LoginPage(page)
  await loginPage.login()

  const ip = config.nic.ips.native[4]

  const provisionPage = new ProvisionPage(page)
  await provisionPage.goto()

  const image = config.images.find(image => image.for.indexOf(brand) !== -1)

  await provisionPage.setAlias(title)
  await provisionPage.selectImage(image.uuid)
  await provisionPage.selectBrand(brand)
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

  const instancePage = new InstanceJoyentMinimalPage(page, uuid)
  await instancePage.isExpectedBrand()
  await instancePage.isRunning()
  await instancePage.stop()
  await instancePage.start()
  if (deleteInstances) {
    await instancePage.deleteInstance()
  }
})
