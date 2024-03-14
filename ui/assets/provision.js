import { $ } from './global'
import { v4 as uuidv4 } from 'uuid'
import {
  EditorView,
  EditorState,
  basicSetup,
  json,
  oneDark
} from '@tpaul/codemirror6-json-rolledup'

// HTMLX needs access to the editors
window.editors = {}

// Ensure an input can't be lower than it's minimium
window.inputBoundToMin = ($element) => {
  const parsedValue = parseInt($element.value, 10)
  const min = $element.min || 0
  if (isNaN(parsedValue)) {
    $element.value = min
  } else {
    $element.value = parsedValue < min ? min : parsedValue
  }
}

// When vCPU is set, if there's nothing already in CPU Cap, pre-fill using vCPUs
// if CPU Cap has a value, set it to at least the value of vCPUs
window.vCpuOnChange = ($element) => {
  const $cpuCap = $('[name=cpu_cap')
  const vCPUs = parseInt($element.value, 10)

  if (isNaN(vCPUs)) {
    return
  }

  const cpuCapValue = vCPUs * 100

  if (!$cpuCap.value) {
    $cpuCap.value = cpuCapValue
  } else {
    const cpuCap = parseInt($cpuCap.value, 10)
    if (!isNaN(cpuCap) || cpuCap < cpuCapValue) {
      $cpuCap.value = cpuCapValue
    }
  }
}

// When cloud-init data is updated, ensure it begins with "#cloud-config\n"
window.cloudInitDataOnChange = ($element) => {
  if (!$element.value) {
    return
  }
  if ($element.value === '#cloud-config') {
    $element.value = '#cloud-config\n'
  } else if (!$element.value.startsWith('#cloud-config\n')) {
    $element.value = `#cloud-config\n${$element.value}`
  }
}

// This function willl interrogate input types for a given <form> and attempt to
// massage them into basic JSON types.
// A data attr named data-enctype="TYPE" can be used on any element with a name
// attr for type hinting this is especially useful for <select> elements that
// have no type attr
function encodeFormParameters ($targets, props = {}) {
  $targets.forEach($element => {
    let value = null
    switch (($element.dataset && $element.dataset.encType) || $element.type) {
      case 'number':
        try {
          value = parseInt($element.value, 10)
        } catch (e) {
          console.error(`Failed parsing number for "${$element.name}"`,
            $element.name, e)
          value = 0
        }
        break
      case 'checkbox':
        if ($element.checked) {
          value = $element.checked
        }
        break
      case 'text':
      case 'textarea':
      case 'hidden':
      default:
        value = $element.value
        break
    }
    if (value || value === 0) {
      props[$element.name] = value
    } else {
      delete props[$element.name]
    }
  })
  return props
}

// Collect all of the data from inputs on the guided form, massage it as
// needed, merge it with the additional properties tab and then write the
// resulting object to the final properties JSON editor.
window.updateEditors = () => {
  const editors = window.editors
  const $form = $('#content form')
  let final = {}
  let additional = {}
  const $targets = $form.querySelectorAll('[name]:not(.nicprop)')
  const props = encodeFormParameters($targets)
  const isHvm = (['bhyve', 'kvm'].indexOf(props.brand) !== -1)
  const nic = {}
  const nicTag = $('[name=nic_tag').value
  const $ipv4Setup = $('[name=ipv4_setup')
  const $ipv6Setup = $('[name=ipv6_setup')
  const $cpuCap = $('[name=cpu_cap')

  if (nicTag) {
    nic.nic_tag = nicTag
    nic.ips = []
  }

  if (nicTag && $ipv4Setup && $ipv4Setup.value === 'static') {
    let ip = $('[name=ipv4_ip').value.trim()
    const gateway = $('[name=ipv4_gateway').value.trim()
    const prefix = $('[name=ipv4_prefix').value.trim()

    if (ip) {
      if (prefix) {
        ip = `${ip}/${prefix}`
      }
      nic.ips.push(ip)
    }

    if (gateway) {
      nic.gateways = [gateway]
    }
  } else if (nicTag && $ipv4Setup && $ipv4Setup.value === 'auto') {
    nic.ips = ['dhcp']
  }

  if (nicTag && $ipv6Setup && $ipv6Setup.value === 'static') {
    let ip = $('[name=ipv6_ip').value.trim()
    const prefix = $('[name=ipv6_prefix').value.trim()
    if (ip) {
      if (prefix) {
        ip = `${ip}/${prefix}`
      }
      nic.ips.push(ip)
    }
  } else if (nicTag && $ipv6Setup && $ipv6Setup.value === 'auto') {
    nic.ips.push('addrconf')
  }

  if (Object.keys(nic).length) {
    if (isHvm) {
      nic.model = 'virtio'
    }
    props.nics = [nic]
  }

  if (props.brand === 'bhyve' && props.image_uuid) {
    props.disks = [
      {
        image_uuid: props.image_uuid,
        boot: true,
        model: 'virtio',
        size: props.primary_disk_size * 1024
      }
    ]
    props.flexible_disk_size = props.primary_disk_size * 1024
    delete props.image_uuid
    delete props.primary_disk_size
  } else if (props.brand === 'kvm' && props.image_uuid) {
    props.disks = [
      {
        image_uuid: props.image_uuid,
        boot: true,
        model: 'virtio'
      }
    ]
    delete props.image_uuid
  } else if (props.disks) {
    delete props.disks
  }

  if (props.resolvers) {
    const resolvers = $('[name=resolvers').value
      .split(',')
      .filter(i => i)
      .map(i => i.trim())
    props.resolvers = resolvers
  }

  if (isHvm && props.vcpus && !props.cpu_cap) {
    props.cpu_cap = props.vcpus * 100
    $cpuCap.value = props.cpu_cap
  }

  if (props.user_script) {
    if (!props.customer_metadata) {
      props.customer_metadata = {}
    }
    props.customer_metadata['user-script'] = props.user_script
  }
  delete props.user_script

  if (props.root_authorized_keys) {
    if (!props.customer_metadata) {
      props.customer_metadata = {}
    }

    props.customer_metadata.root_authorized_keys = props.root_authorized_keys

    if (!isHvm) {
      if (!props.internal_metadata) {
        props.internal_metadata = {}
      }
      props.internal_metadata['operator-script'] = `#!/usr/bin/bash
if ! [[ -d /root/.ssh ]]; then
  mkdir -p /root/.ssh
  chmod 0700 /root/.ssh
fi
/usr/sbin/mdata-get root_authorized_keys > /root/.ssh/authorized_keys
/usr/sbin/mdata-delete operator-script`
    }
  }
  delete props.root_authorized_keys

  if (props.cloudinit_data) {
    if (!props.customer_metadata) {
      props.customer_metadata = {}
    }
    props.customer_metadata['cloud-init:user-data'] = props.cloudinit_data
  }
  delete props.cloudinit_data

  if (props.root_pw) {
    props.internal_metadata = {
      root_pw: props.root_pw
    }
    delete props.root_pw
  }

  if (props.alias) {
    props.hostname = props.alias
  }

  try {
    const content = editors.additional.state.doc.toString()
    if (content) {
      additional = JSON.parse(content)
      editors.additional.dispatch({
        changes: {
          from: 0,
          to: editors.additional.state.doc.length,
          insert: JSON.stringify(additional, null, 2)
        }
      })
    } else {
      editors.additional.dispatch({
        changes: {
          from: 0,
          to: editors.additional.state.doc.length,
          insert: '{}'
        }
      })
    }
    switch ($('#merge').value) {
      case 'replace':
        final = Object.assign({}, props, additional)
        break
      case 'ignore':
        final = Object.assign({}, additional)
        break
      case 'extend':
      default:
        final = Object.assign({}, additional, props)
        break
    }
  } catch (e) {
    window.alert('Failed to serialize JSON in additional properties.')
    console.error(e)
    return false
  }

  editors.final.dispatch({
    changes: {
      from: 0,
      to: editors.final.state.doc.length,
      insert: JSON.stringify(final, null, 2)
    }
  })

  return true
}

window.getFinalEditor = () => {
  window.updateEditors()
  // We need a UUID on the instance payload so we can keep track of it
  // If the user didn't add one generate it here so we know what it is.
  let payload = window.editors.final.state.doc.toString()
  try {
    const instancePayload = JSON.parse(payload)
    if (!instancePayload.uuid) {
      instancePayload.uuid = uuidv4()
      payload = JSON.stringify(instancePayload)
    }
  } catch (e) {
    console.error(e)
    window.alert('Failed parsing final properties for instance.')
    return
  }

  return payload
}

export const setupProvisioningForm = () => {
  const $guidedTab = $('#guided-tab')
  const $additionalTab = $('#additional-tab')
  const $finalTab = $('#final-tab')
  const $guidedButton = $('#guided-button')
  const $additionalButton = $('#additional-button')
  const $finalButton = $('#final-button')
  const $validateButton = $('#validate-button')
  const $actionButtons = $('#action-buttons')
  const $editorTabs = [$additionalTab, $finalTab]
  const $tabs = [$guidedTab, ...$editorTabs]
  const $buttons = [$guidedButton, $additionalButton, $finalButton]
  const active = 'active-editor-tab'
  const inactive = 'inactive-editor-tab'
  const updateEditors = window.updateEditors
  let editorsSetup = false

  function clearStyle ($tabs, $buttons) {
    $tabs.forEach($element => $element.classList.add('hidden'))
    $buttons.forEach($element => {
      $element.classList.remove(active)
      $element.classList.add(inactive)
    })
  }

  // Setup each CodeMirror editor
  $editorTabs.forEach($tab => {
    const editors = window.editors
    const { name } = $tab.dataset
    if ($tab.querySelector('.cm-editor')) {
      editorsSetup = true
      return
    }
    const parent = $tab.querySelector('.editor-wrapper')
    const extensions = [basicSetup, json(), oneDark]
    if ($tab.dataset.readOnly) {
      extensions.push(EditorState.readOnly.of(true))
    }
    editors[name] = new EditorView({
      extensions,
      parent
    })
  })

  if (editorsSetup) {
    // Don't need to re-create event handlers
    return
  }

  $finalButton.addEventListener('click', () => {
    if (updateEditors()) {
      $validateButton.click()
      clearStyle($tabs, $buttons)
      $finalTab.classList.remove('hidden')
      $finalButton.classList.remove(inactive)
      $finalButton.classList.add(active)
    }
  })

  $additionalButton.addEventListener('click', () => {
    if (updateEditors()) {
      clearStyle($tabs, $buttons)
      $additionalTab.classList.remove('hidden')
      $additionalButton.classList.remove(inactive)
      $additionalButton.classList.add(active)
    }
  })

  $guidedButton.addEventListener('click', () => {
    if (updateEditors()) {
      clearStyle($tabs, $buttons)
      $guidedTab.classList.remove('hidden')
      $guidedButton.classList.remove(inactive)
      $guidedButton.classList.add(active)
    }
  })

  // Show/Hide major inputs and controls when an image_uuid is selected
  $guidedTab.addEventListener('change', e => {
    if (e.target.id === 'image_uuid') {
      if (e.target.value) {
        $additionalButton.classList.remove('hidden')
        $finalButton.classList.remove('hidden')
        $actionButtons.classList.remove('hidden')
      } else {
        $additionalButton.classList.add('hidden')
        $finalButton.classList.add('hidden')
        $actionButtons.classList.add('hidden')
      }
    }
  })
}
