use leptos::*;
use leptos::nonce::use_nonce;
use leptos::prelude::{ElementChild, GlobalAttributes};

#[component]
pub fn PopupScript() -> impl IntoView {
    let nonce = use_nonce();
    view! {
        <script nonce=nonce>
            {r#"
  console.log('Popup script starting...');

  let isInitialized = false;

  function handlePopupHash() {
    const hash = window.location.hash;
    console.log('=== handlePopupHash called ===');
    console.log('Current hash:', hash);

    const allPopups = document.querySelectorAll('.popup');
    console.log('Found popups:', allPopups.length);

    // Скрываем все popup'ы
    allPopups.forEach(popup => {
      popup.style.display = 'none';
    });

    // Показываем нужный popup
    if (hash && hash.startsWith('#popup-')) {
      const targetPopup = document.querySelector(hash);
      console.log('Target popup:', targetPopup ? targetPopup.id : 'NOT FOUND');

      if (targetPopup) {
        targetPopup.style.display = 'flex';
        targetPopup.style.position = 'fixed';
        targetPopup.style.inset = '0';
        targetPopup.style.zIndex = '99999';
        targetPopup.style.background = 'rgba(0, 0, 0, 0.9)';
        targetPopup.style.justifyContent = 'center';
        targetPopup.style.alignItems = 'center';
        console.log('Popup opened!');
      }
    }
  }

  function initPopupHandlers() {
    if (isInitialized) return;

    const popups = document.querySelectorAll('.popup');
    console.log('Initializing popup handlers, found:', popups.length);

    if (popups.length > 0) {
      isInitialized = true;

      // Добавляем обработчик hashchange
      window.addEventListener('hashchange', handlePopupHash);

      // Проверяем текущий hash
      handlePopupHash();

      console.log('Popup handlers initialized!');
    }
  }

  function startObserving() {
    // Проверяем что body существует
    if (!document.body) {
      console.log('Body not ready, waiting...');
      setTimeout(startObserving, 50);
      return;
    }

    console.log('Body ready, starting MutationObserver');

    // MutationObserver следит за появлением popup'ов
    const observer = new MutationObserver(function(mutations) {
      for (let mutation of mutations) {
        if (mutation.addedNodes.length > 0) {
          const hasPopup = Array.from(mutation.addedNodes).some(node =>
            node.classList && node.classList.contains('popup')
          );

          if (hasPopup || document.querySelectorAll('.popup').length > 0) {
            console.log('Popups detected in DOM!');
            initPopupHandlers();
            break;
          }
        }
      }
    });

    // Наблюдаем за изменениями в DOM
    observer.observe(document.body, {
      childList: true,
      subtree: true
    });

    console.log('MutationObserver started');

    // Пробуем инициализировать сразу
    initPopupHandlers();

    // Пробуем через интервалы
    let attempts = 0;
    const checkInterval = setInterval(function() {
      attempts++;
      console.log('Check attempt:', attempts, 'popups:', document.querySelectorAll('.popup').length);
      initPopupHandlers();

      if (isInitialized || attempts > 30) {
        clearInterval(checkInterval);
        console.log('Stopped checking, initialized:', isInitialized);
      }
    }, 200);
  }

  // Запускаем когда DOM готов
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', startObserving);
  } else {
    startObserving();
  }
  window.handlePopupHash = handlePopupHash;
  console.log('Popup script loaded');
"#}
        </script>
    }
}