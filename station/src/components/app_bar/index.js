import { LitElement, css, html } from 'lit'

export class AppBar extends LitElement {
  static get properties() {
    return {
    }
  }

  constructor() {
    super()
  }

  render() {
    return html`
      <div>
        Meow
      </div>
    `
  }

  _onClick() {
    this.count++
  }

  static get styles() {
    return css`
      :host {
      }
    `
  }
}

window.customElements.define('app-bar', AppBar);
