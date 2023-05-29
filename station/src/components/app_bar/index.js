import { LitElement, css, html } from 'lit';

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
      <div class="top_bar" @style="meow">
        ${`Meow`}
      </div>
    `
  }

  _onClick() {
    this.count++
  }

  static get styles() {
    return css`
      :host {
        background: red;
        visibili
      }
      .top_bar {
        display: block;
        position: absolute;
      }
    `
  }
}

window.customElements.define('app-bar', AppBar);
