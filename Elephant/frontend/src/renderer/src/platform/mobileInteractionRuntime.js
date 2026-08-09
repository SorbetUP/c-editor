import '../mobile-recovery.css'
import '../mobile-density.css'
import '../addons/officialAddonCatalogBridge'
import { useNavigationStore } from 'elephant-front/stores/navigationStore'
import { useSearchStore } from 'elephant-front/stores/searchStore'
import { useVaultStore } from 'elephant-front/stores/vaultStore'

const MOBILE_BACK_STATE = '__elephantMobileBackState'
const CLOSE_DRAWER_SELECTOR = '.en-mobile-scrim.visible'

const isMobileViewport = (target = globalThis) => Boolean(
  target.matchMedia?.('(max-width: 760px), (hover: none) and (pointer: coarse)').matches
)

const closeDrawer = (target = globalThis) => {
  const scrim = target.document?.querySelector?.(CLOSE_DRAWER_SELECTOR)
  if (!scrim || typeof scrim.click !== 'function') return false
  scrim.click()
  return true
}

const armMobileBackState = (target = globalThis) => {
  const current = target.history?.state || {}
  if (current[MOBILE_BACK_STATE]) return
  target.history?.pushState?.({ ...current, [MOBILE_BACK_STATE]: true }, '', target.location?.href)
}

const installAndroidBackNavigation = (target = globalThis) => {
  let handlingBack = false

  const onPopState = async () => {
    if (!isMobileViewport(target) || handlingBack) return
    handlingBack = true
    let handled = false

    try {
      const backEvent = new CustomEvent('elephantnote:android-back', { cancelable: true })
      if (!target.dispatchEvent(backEvent)) {
        handled = true
      } else {
        const settingsClose = target.document.querySelector('.en-settings-close')
        if (settingsClose) {
          settingsClose.click()
          handled = true
        } else {
          const searchStore = useSearchStore()
          if (searchStore.isOpen) {
            searchStore.close()
            handled = true
          } else {
            const vaultStore = useVaultStore()
            const navigationStore = useNavigationStore()
            if (vaultStore.openedNotePath) {
              vaultStore.closeNote()
              if (target.document.querySelector(CLOSE_DRAWER_SELECTOR)) closeDrawer(target)
              handled = true
            } else if (target.document.querySelector(CLOSE_DRAWER_SELECTOR)) {
              handled = closeDrawer(target)
            } else {
              const previous = navigationStore.back()
              if (previous) {
                await vaultStore.navigateTo(previous)
                handled = true
              }
            }
          }
        }
      }
    } catch (error) {
      console.error('[mobile-navigation] Android back handling failed', error)
    } finally {
      handlingBack = false
      if (handled) target.setTimeout(() => armMobileBackState(target), 0)
    }
  }

  target.addEventListener('popstate', onPopState)
  armMobileBackState(target)
  return () => target.removeEventListener('popstate', onPopState)
}

export const installMobileInteractionRuntime = (target = globalThis) => {
  if (!target?.document || target.__ELEPHANT_MOBILE_INTERACTIONS_INSTALLED__) return false
  target.__ELEPHANT_MOBILE_INTERACTIONS_INSTALLED__ = true
  const removeBackNavigation = installAndroidBackNavigation(target)

  target.__ELEPHANT_MOBILE_INTERACTIONS_DISPOSE__ = () => {
    removeBackNavigation()
    target.__ELEPHANT_MOBILE_INTERACTIONS_INSTALLED__ = false
  }

  console.info('[mobile-navigation] Android back navigation installed; drawer gestures are Vue-owned')
  return true
}

installMobileInteractionRuntime()
