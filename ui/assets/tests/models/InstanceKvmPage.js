const InstanceHvmPage = require('./InstanceHvmPage')

class InstanceKvmPage extends InstanceHvmPage {
  brand = 'kvm'
}

module.exports = InstanceKvmPage
