const Page = require('./Page')
const { expect } = require('@playwright/test')
const { instanceCreateTimeout } = require('../config')

class ProvisionPage extends Page {
  constructor (page) {
    super(page, '/provision', 'Create Instance')
  }

  async setAlias (alias) {
    this.alias = alias
    await this.page.locator('#alias').fill(alias)
  }

  async setRam (ram) {
    await this.page.locator('#ram').fill(String(ram))
  }

  async setSSHKey (key) {
    await this.page.locator('#root_authorized_keys').fill(key)
  }

  async setQuota (quota) {
    await this.page.locator('#quota').fill(String(quota))
  }

  async setPrimaryDiskSize (gb) {
    await this.page.locator('#primary_disk_size').fill(String(gb))
  }

  async setCpus (cpus) {
    await this.page.locator('#vcpus').fill(String(cpus))
  }

  async setRootPassword (password) {
    await this.page.locator('#root_pw').fill(password)
  }

  async setDelegateDataset () {
    await this.page.locator('#delegate_dataset').check()
  }

  async selectNicTag (tag) {
    await this.changeSelect('#nic_tag', tag)
    // Wait for nic subform to populate
    await expect(this.page.locator('#resolvers')).toHaveCount(1)
  }

  async setResolvers (resolvers) {
    await this.page.locator('#resolvers').fill(resolvers)
  }

  async selectIPv4Setup (setup) {
    await this.changeSelect('#ipv4_setup', setup)
  }

  async setIPv4Address (address) {
    await this.page.locator('#ipv4_ip').fill(address)
  }

  async selectIpv4Prefix (prefix) {
    return this.changeSelect('#ipv4_prefix', prefix)
  }

  async setIpv4Gateway (gateway) {
    await this.page.locator('#ipv4_gateway').fill(gateway)
  }

  async selectIPv6Setup (setup) {
    return this.changeSelect('#ipv6_setup', setup)
  }

  async selectImage (uuid) {
    await this.changeSelect('#image_uuid', uuid)
  }

  async selectBrand (brand) {
    await this.changeSelect('#brand', brand)
  }

  async isBrand (brand) {
    await expect(await this.page.locator('#brand')).toHaveValue(brand)
  }

  async validate () {
    await this.page.locator('#validate-button').click()
    const [locator, close] = this.getNotification('')
    await expect(locator).toContainText('Valid Instance Details')
    await close.click()
  }

  async create () {
    await this.page.locator('#create-button').click()
  }

  async createSuccess () {
    const locator = this.page.locator('#modal-title')
    await expect(locator).toContainText('Provision successful',
      instanceCreateTimeout)
  }

  async viewDetails () {
    const detailsButton = await this.page.getByRole('button', { name: 'Instance Details' })
    await detailsButton.click()
    await expect(this.page.locator('#prop-alias')).toHaveValue(this.alias)
    const input = this.page.locator('#prop-uuid')
    return await input.evaluate(element => element.value)
  }
}

module.exports = ProvisionPage
