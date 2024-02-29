const Page = require('./Page')
const InstanceNativePage = require('./InstanceNativePage')

class InstanceLXPage extends InstanceNativePage {
  brand = 'lx'
  constructor (page, uuid) {
    super(page, uuid)
  }
}

module.exports = InstanceLXPage
