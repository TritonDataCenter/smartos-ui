// helpers
function $(selector) {
  return document.querySelector(selector)
}

function $$(selector) {
  return document.querySelectorAll(selector)
}

// Handle showing/hiding notifications
document.addEventListener('DOMContentLoaded', () => {

  const $notify = $('#notify')

  function hideNotify() {
      $notify.classList.remove('opacity-100')
      $notify.classList.add('opacity-0')
  }

  function showNotify() {
      $notify.classList.remove('opacity-0')
      $notify.classList.add('opacity-100')
  }

  $('#notify-close').addEventListener('click', hideNotify)

  function notify(notification, timeout=10000) {
    console.log(notification)
    if (!notification || !notification.heading || !notification.body) {
      // nothing to do here
      return
    }

    const $heading = $('#notify-body')
    const $body = $('#notify-heading')
    const icons = {
      ok: $('#notifiy-icon-ok'),
      error: $('#notifiy-icon-error'),
      info: $('#notifiy-icon-info')
    }
    const icon = notification.icon || 'ok'

    $heading.innerText = notification.heading
    $body.innerText = notification.body

    icons[icon].classList.remove('hidden')

    showNotify()

    setTimeout(() => {
      hideNotify()
      icons.ok.classList.add('hidden')
      icons.error.classList.add('hidden')
      icons.info.classList.add('hidden')
    }, timeout)
  }

  // Update the sidebar to show the currently selected view.
  // This will occur automatically on a full page reload but we must handle it
  // on our own when navigating via HTMX. Perhaps we can instead make the
  // sidebar navigation a template and re-render it server side on a request?
  htmx.on('htmx:pushedIntoHistory', ({detail: {path}}) => {
    const active = ['bg-gray-800', 'text-white']
    const inactive =['text-gray-400', 'hover:text-white', 'hover:bg-gray-800']
    $$('.main-nav').forEach(element => {
      let match = element.getAttribute('hx-get') === path
      element.classList.remove(...match ? inactive : active)
      element.classList.add(...match ? active : inactive)
    })
  })

  htmx.on('htmx:beforeOnLoad', e => {
    console.log('htmx:beforeOnLoad', document.location.pathname, e.detail)
  })

  // Long running requests such as importing images or creating an instance
  // may send their response when the user has navigated to another view within
  // the app which will then redirect them to the path in the HX-Location header
  // which may be undesirable. To account for this, these long running requests
  // send back a list of paths that are acceptable for redirecting. If the user
  // is not currently at one of these paths a notification will be shown.
  htmx.on('htmx:beforeRequest', e => {
    const {detail: {etc: {values}}} = e
    if (values) {
      let {longRunning, allowedPaths, notification, alwaysNotify} = values
      let {pathname} = document.location;
      if (longRunning && allowedPaths.indexOf(pathname) == -1) {
        e.preventDefault()
        return notify(notification)
      } else if (longRunning && alwaysNotify) {
        notify(notification)
      }
    }
  })
})