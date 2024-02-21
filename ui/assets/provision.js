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

// Will interrogate input types for a given <form> and attempt to massage
// them into basic JSON types.
// A data attr named data-enctype="TYPE" can be used on any element with a name
// attr for preferred type hinting this is especially useful for <select>
// elements that have no type attr
function encodeFormParameters ($targets, props = {}) {
  $targets.forEach($element => {
    let value
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
      default:
        value = $element.value
        break
    }
    if (value) {
      props[$element.name] = value
    } else {
      delete props[$element.name]
    }
  })
  return props
}

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
    nic.ips = ['dhcp', 'addrconf']
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
    if (nic.ips.indexOf('dhcp') === -1) {
      nic.ips.push(...['dhcp', 'addrconf'])
    }
  }

  if (Object.keys(nic).length) {
    if (isHvm) {
      nic.model = 'virtio'
    }
    props.nics = [nic]
  }

  if (isHvm && props.image_uuid) {
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

  // how to compute?
  // flexible_disk_size:
  //
  //     This sets an upper bound for the amount of space that a bhyve instance
  //     may use for its disks and snapshots of those disks. If this value is not
  //     set, it will not be possible to create snapshots of the instance.
  //
  //     This value must be at least as large as the sum of all of the
  //     disk.*.size values.
  //
  //     type: integer (number of MiB)
  //     vmtype: bhyve
  //     listable: yes
  //     create: yes
  //     update: yes (live update)
  //
  // if (props.brand === 'bhyve') {
  //   props.flexible_disk_size = true
  // }

  // Specifying bootrom, causes vmadm validate to complain about image size...
  // if (props.brand === 'bhyve') {
  //   props.bootrom = 'uefi'
  // }

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
  // vmadm will generate one if it's not provided so if the user didn't add one
  // generate it here so we know what it is.
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
}
