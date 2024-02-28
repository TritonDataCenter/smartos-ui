const Page = require('./Page')
const InstanceNativePage = require('./InstanceNativePage')

class InstanceJoyentMinimalPage extends InstanceNativePage {
  brand = 'joyent-minimal'
  constructor (page, uuid) {
    super(page, uuid)
  }
}

module.exports = InstanceJoyentMinimalPage
