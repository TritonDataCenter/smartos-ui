/*
 *
 * This test assumes first-run.spec.js has already been executed successfully
 * or the required images have already been imported.
 *
 * This test will provision and then delete bhyve and kvm instances.
 *
 */
const { test, expect } = require('@playwright/test')
const config = require('./config')
const LoginPage = require('./models/LoginPage')
const ProvisionPage = require('./models/ProvisionPage')
const InstanceBhyvePage = require('./models/InstanceBhyvePage')
const InstanceKvmPage = require('./models/InstanceKvmPage')

// For additional manual testing, set to false so instances can be inspected
const deleteInstances = true

test.use({
  ignoreHTTPSErrors: true
})

test.describe.configure({ mode: 'serial' })

test('Create Bhyve (all defaults)', async ({ page }, { title }) => {
  test.setTimeout(config.instanceCreateTimeout.timeout)
  const brand = 'bhyve'
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

  const instancePage = new InstanceBhyvePage(page, uuid)
  await instancePage.isExpectedBrand()
  await instancePage.isRunning()
  await instancePage.stop()
  await instancePage.start()
  if (deleteInstances) {
    await instancePage.deleteInstance()
  }
})

test('Create KVM (all defaults)', async ({ page }, { title }) => {
  test.setTimeout(config.instanceCreateTimeout.timeout)
  const brand = 'kvm'
  const loginPage = new LoginPage(page)
  await loginPage.login()

  const provisionPage = new ProvisionPage(page)
  await provisionPage.goto()

  const image = config.images.find(image => image.for.indexOf(brand) !== -1)

  await provisionPage.setAlias(title)
  await provisionPage.selectImage(image.uuid)
  await provisionPage.selectBrand(brand)
  await provisionPage.isBrand(brand)

  // KVM needs to wait for bootrom element to dissapear
  await expect(page.locator('#bootrom')).toHaveCount(0)

  await provisionPage.validate()
  await provisionPage.create()
  await provisionPage.createSuccess()
  const uuid = await provisionPage.viewDetails()

  const instancePage = new InstanceKvmPage(page, uuid)
  await instancePage.isExpectedBrand()
  await instancePage.isRunning()
  await instancePage.stop()
  await instancePage.start()
  if (deleteInstances) {
    await instancePage.deleteInstance()
  }
})

test('Create Bhyve (1024MiB RAM, 16GiB Disk, 2vCPUs, Ipv4 Static)', async ({ page }, { title }) => {
  test.setTimeout(config.instanceCreateTimeout.timeout)
  const brand = 'bhyve'
  const loginPage = new LoginPage(page)
  await loginPage.login()

  const ip = config.nic.ips.hvm[0]

  const provisionPage = new ProvisionPage(page)
  await provisionPage.goto()

  const image = config.images.find(image => image.for.indexOf(brand) !== -1)

  await provisionPage.setAlias(title)
  await provisionPage.selectImage(image.uuid)
  await provisionPage.isBrand(brand)
  await provisionPage.setRam(1024)
  await provisionPage.setPrimaryDiskSize(16)
  await provisionPage.setCpus(2)
  await provisionPage.setRootPassword('test123')
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

  const instancePage = new InstanceBhyvePage(page, uuid)
  await instancePage.isExpectedBrand()
  await instancePage.isRunning()
  await instancePage.stop()
  await instancePage.start()
  if (deleteInstances) {
    await instancePage.deleteInstance()
  }
})
