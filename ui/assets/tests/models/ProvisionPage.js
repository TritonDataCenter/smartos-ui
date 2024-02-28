const Page = require('./Page')
const { expect } = require('@playwright/test')
const {instanceCreateTimeout} = require('../config')

class ProvisionPage extends Page {
  constructor (page) {
    super(page, '/provision', 'Create Instance')
  }

  async setAlias(alias) {
    this.alias = alias
    await this.page.locator('#alias').fill(alias)
  }

  async setRam(ram) {
    await this.page.locator('#ram').fill(String(ram))
  }

  async setSSHKey(key) {
    await this.page.locator('#root_authorized_keys').fill(key)
  }

  async setQuota(quota) {
    await this.page.locator('#quota').fill(String(quota))
  }

  async setDelegateDataset() {
    await this.page.locator('#delegate_dataset').check()
  }

  async selectNicTag(tag) {
    await this.page.locator('#nic_tag').selectOption({value: tag})
  }

  async setResolvers(resolvers) {
    await this.page.locator('#resolvers').fill(resolvers)
  }

  async selectIPv4Setup(setup) {
    await this.page.locator('#ipv4_setup').selectOption({value: setup})
  }

  async setIPv4Address(address) {
    await this.page.locator('#ipv4_ip').fill(address)
  }

  async selectIpv4Prefix(prefix) {
    await this.page.locator('#ipv4_prefix').selectOption({value: prefix})
  }

  async setIpv4Gateway(gateway) {
    await this.page.locator('#ipv4_gateway').fill(gateway)
  }

  async selectIPv6Setup(setup) {
    await this.page.locator('#ipv6_setup').selectOption({value: setup})
  }

  async selectImage(uuid) {
    await this.page.locator('#image_uuid').selectOption({value: uuid})
  }

  async selectBrand(brand) {
    await this.page.locator('#brand').selectOption({value: brand})
  }

  async isBrand(brand) {
    await expect(await this.page.locator('#brand')).toHaveValue(brand)
  }

  async validate() {
    await this.page.locator('#validate-button').click()
    const validateResult = this.page.locator('#validate-result')
    await expect(validateResult).toContainText('Valid Instance Details')
  }

  async create() {
    await this.page.locator('#create-button').click()
    const createResult = this.page.locator('#modal-title')
  }

  async createSuccess() {
    const locator = this.page.locator('#modal-title')
    await expect(locator).toContainText('Provision successful',
      instanceCreateTimeout)
  }

  async viewDetails() {
    const detailsButton = await this.page.getByRole('button', { name: 'Instance Details' })
    await detailsButton.click()
    await expect(this.page.locator('#prop-alias')).toHaveValue(this.alias)
    const input = this.page.locator('#prop-uuid')
    return await input.evaluate(element => element.value)
  }
}

module.exports = ProvisionPage