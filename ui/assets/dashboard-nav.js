// Update the sidebar to show the currently selected view.
// This will occur automatically on a full page reload but we must handle it
// on our own when navigating via HTMX. Perhaps we can instead make the sidebar
// navigation a template and re-render it server side on a request?
htmx.on('htmx:pushedIntoHistory', ({detail: {path}}) => {
  const active = ['bg-gray-800', 'text-white']
  const inactive =['text-gray-400', 'hover:text-white', 'hover:bg-gray-800']
  document.querySelectorAll('.main-nav').forEach(element => {
    let match = element.getAttribute('hx-get') === path
    element.classList.remove(...match ? inactive : active)
    element.classList.add(...match ? active : inactive)
  })
})