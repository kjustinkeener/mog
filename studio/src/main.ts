import { mount } from 'svelte'
import './app.css'
import App from './App.svelte'
import Installer from './Installer.svelte'
import { invoke } from '@tauri-apps/api/core'
import { initTheme } from './lib/theme'

export interface SetupState {
  needs_setup: boolean
  installed: boolean
  has_engine: boolean
  version: string
  build_date: string
  install_dir: string
}

const target = document.getElementById('app')!

async function boot() {
  initTheme()
  const isInstaller = window.location.hash === '#installer'
  try {
    const s = await invoke<SetupState>('setup_state')
    if (isInstaller || s.needs_setup) {
      return mount(Installer, { target, props: { setup: s } })
    }
  } catch {
    // Not running under Tauri (or the command failed): boot the app.
  }
  return mount(App, { target })
}

export default boot()
