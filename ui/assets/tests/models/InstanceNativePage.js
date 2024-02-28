const Page = require('./Page')
const InstancePage = require('./InstancePage')

class InstanceNativePage extends InstancePage {
  constructor (page, uuid) {
    super(page, uuid)
  }
}

module.exports = InstanceNativePage
