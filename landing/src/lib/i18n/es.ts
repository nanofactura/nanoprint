import type { Messages } from './types';

// Spanish as written in Argentina (voseo), like the app. Text between backticks renders as code.
export const es: Messages = {
  meta: { locale: 'es_AR' },
  nav: {
    features: 'Funciones',
    download: 'Descargar',
    developers: 'Desarrolladores',
    faq: 'Preguntas',
    github: 'GitHub',
    otherLanguage: 'English',
    languageLabel: 'Read in English',
    skip: 'Saltar al contenido',
    menu: 'Menú',
  },
  download: {
    generic: 'Descargar NanoPrint',
    windows: 'Descargar para Windows',
    mac: 'Descargar para macOS',
    linux: 'Descargar para Linux',
    otherPlatforms: 'Otros sistemas',
    previousVersions: 'Versiones anteriores',
    desktopOnly: 'NanoPrint es para computadoras con Windows, macOS o Linux.',
  },
  home: {
    title: 'NanoPrint: imprimí tickets desde la web en tu impresora térmica',
    description:
      'Programa gratuito y de código abierto para Windows, macOS y Linux que imprime tickets ESC/POS desde tu sistema web en impresoras térmicas, con permiso por sitio. Compatible con qz-tray.js.',
    hero: {
      title: 'Imprimí tickets desde la web, directo a tu impresora térmica',
      lead: 'NanoPrint es un programa liviano que conecta tu sistema web con las impresoras de tickets de la computadora. Cada sitio pide permiso antes de imprimir.',
      platforms: 'Windows · macOS · Linux',
      compatible: 'Compatible con qz-tray.js',
      videoLabel:
        'Ver de nuevo la animación: una impresora térmica imprime un ticket de NanoPrint.',
    },
    features: {
      title: 'Hecho para el mostrador',
      lead: 'Lo que necesita un comercio: que el ticket salga rápido, sin configuraciones raras y sin riesgos.',
      items: [
        {
          title: 'Multiplataforma',
          text: 'El mismo sistema web imprime igual en Windows, macOS y Linux, con las impresoras que ya tenés instaladas.',
          points: [
            'Windows 10 y 11',
            'macOS 10.15 o posterior, Intel y Apple Silicon',
            'Ubuntu, Debian, Fedora y otras distribuciones',
          ],
        },
        {
          title: 'Liviano',
          text: 'Un instalador de pocos megas que se queda en la bandeja del sistema y arranca con la computadora.',
          points: [
            'No necesita Java ni otros programas',
            'En Windows se instala sin permisos de administrador',
            'En español y en inglés',
          ],
        },
        {
          title: 'Seguro',
          text: 'Ningún sitio imprime sin tu permiso, y lo que imprimís no sale de tu computadora.',
          points: [
            'Permiso por sitio, revocable en cualquier momento',
            'Solo acepta conexiones de esta misma computadora',
            'Nunca guarda el contenido de lo impreso',
          ],
        },
      ],
    },
    how: {
      title: 'Cómo funciona',
      items: [
        {
          title: 'Instalá NanoPrint',
          text: 'Queda en la bandeja del sistema, junto al reloj, y se inicia con la computadora.',
        },
        {
          title: 'Abrí tu sistema web',
          text: 'La primera vez que un sitio quiere imprimir, NanoPrint te pregunta si lo permitís.',
        },
        {
          title: 'Imprimí',
          text: 'El ticket sale directo en la impresora, sin el diálogo de impresión del navegador.',
        },
      ],
    },
    compat: {
      title: '¿Tu sistema ya usa QZ Tray?',
      text: 'NanoPrint entiende el mismo protocolo que qz-tray.js para la impresión directa (raw). Si tu sistema web ya imprime tickets con qz-tray.js, en general funciona sin cambiar código.',
      link: 'Detalles para desarrolladores',
      note: 'QZ Tray es una marca de QZ Industries, LLC. NanoPrint es un proyecto independiente, sin relación con QZ Industries.',
    },
    printers: {
      title: 'Impresoras compatibles',
      text: 'Impresoras térmicas de tickets que entienden ESC/POS, de 58 u 80 mm, conectadas por USB o por red e instaladas en el sistema. ESC/POS es el lenguaje de la gran mayoría de las impresoras de tickets.',
      driversTitle: '¿Hace falta un driver?',
      drivers: [
        {
          os: 'Windows',
          text: 'La impresora tiene que aparecer en Configuración → Impresoras y escáneres. Sirve el driver del fabricante y, si no hay, el driver «Generic / Text Only» que trae Windows: NanoPrint envía los comandos tal cual.',
        },
        {
          os: 'macOS',
          text: 'Agregala en Configuración del Sistema → Impresoras y escáneres con cualquier driver. NanoPrint le envía los datos sin procesar, así que el driver no los modifica.',
        },
        {
          os: 'Linux',
          text: 'Agregala en la configuración de impresoras (CUPS) con el driver del fabricante o como impresora genérica. NanoPrint imprime sin filtros, en modo raw.',
        },
      ],
      note: 'NanoPrint no instala drivers: así no necesita permisos de administrador. Las impresoras virtuales (PDF, XPS, fax) no se ofrecen a los sitios.',
    },
    downloads: {
      title: 'Descargar NanoPrint',
      lead: 'Gratis y sin registro. Elegí tu sistema.',
      requirements: 'Requisitos',
      install: 'Instalación',
      windows: {
        name: 'Windows',
        requirements: 'Windows 10 u 11 de 64 bits',
        files: [{ label: 'Instalador (.exe)', download: 'windows' }],
        steps: [
          'Abrí el archivo descargado.',
          'Si aparece «Windows protegió su PC», hacé clic en «Más información» y después en «Ejecutar de todas formas». El aviso aparece porque el instalador todavía no tiene una firma digital comercial.',
          'Seguí el asistente. No pide permisos de administrador.',
        ],
      },
      mac: {
        name: 'macOS',
        requirements: 'macOS 10.15 o posterior, Intel y Apple Silicon',
        files: [{ label: 'Imagen de disco (.dmg)', download: 'mac' }],
        steps: [
          'Abrí el archivo .dmg y arrastrá NanoPrint a Aplicaciones.',
          'La primera vez, macOS avisa que no puede verificar al desarrollador. Abrí Configuración del Sistema → Privacidad y seguridad y hacé clic en «Abrir igualmente».',
          'Confirmá con tu contraseña. Las próximas veces abre directo.',
        ],
      },
      linux: {
        name: 'Linux',
        requirements: '64 bits: Ubuntu 22.04, Debian 12, Fedora 38 o posteriores',
        files: [
          { label: '.deb (Ubuntu, Debian)', download: 'deb' },
          { label: '.rpm (Fedora, openSUSE)', download: 'rpm' },
          { label: 'AppImage (otras)', download: 'appImage' },
        ],
        steps: [
          'Ubuntu o Debian: `sudo apt install ./NanoPrint_amd64.deb`',
          'Fedora: `sudo dnf install ./NanoPrint_x86_64.rpm`',
          'AppImage: `chmod +x NanoPrint_amd64.AppImage` y abrilo.',
        ],
      },
      afterTitle: 'Después de instalar',
      afterText:
        'NanoPrint te da la bienvenida y te ofrece activar los sitios seguros (https) con un clic. Después queda en la bandeja del sistema, junto al reloj.',
      previous: 'Versiones anteriores y novedades de cada versión',
      verifyTitle: 'Verificar el instalador',
      verifyText:
        'Cada instalador se compila en GitHub a partir del código publicado y lleva una certificación de origen. Con la herramienta `gh` de GitHub podés comprobarlo: `gh attestation verify NanoPrint_x64-setup.exe --repo nanofactura/nanoprint`',
    },
    openSource: {
      title: 'Código abierto',
      text: 'NanoPrint es software libre bajo la licencia GPL-3.0. Todo el código está en GitHub: podés revisarlo, reportar problemas o colaborar.',
      button: 'Ver en GitHub',
    },
    faq: {
      title: 'Preguntas frecuentes',
      items: [
        {
          q: '¿NanoPrint es gratis?',
          a: 'Sí, para uso personal y comercial. Es software libre bajo la licencia GPL-3.0.',
        },
        {
          q: '¿Funciona con sistemas que usan QZ Tray?',
          a: 'Sí, para la impresión directa de tickets: NanoPrint implementa el protocolo de qz-tray.js. Las funciones de QZ Tray que no son impresión directa (archivos, puertos serie, USB, impresión de PDF o HTML) no están disponibles.',
        },
        {
          q: '¿Qué impresoras funcionan?',
          a: 'Impresoras térmicas de tickets ESC/POS de 58 u 80 mm instaladas en el sistema, por USB o por red. Desde el panel de NanoPrint podés imprimir una página de prueba y elegir qué impresoras ve cada sitio.',
        },
        {
          q: '¿Tengo que instalar un driver especial?',
          a: 'No. La impresora tiene que estar instalada en el sistema operativo, con el driver del fabricante o uno genérico. NanoPrint envía los comandos tal cual.',
        },
        {
          q: '¿Una página puede imprimir sin que me dé cuenta?',
          a: 'No. Cada sitio tiene que pedir permiso y vos decidís. NanoPrint solo acepta conexiones de esta misma computadora, y el permiso se puede revocar desde el panel.',
        },
        {
          q: '¿NanoPrint guarda lo que imprimo?',
          a: 'No. El historial registra durante 30 días la fecha, el sitio, la impresora y el resultado de cada trabajo, nunca su contenido. NanoPrint no envía datos a internet.',
        },
        {
          q: '¿Por qué Windows o macOS muestran un aviso al instalar?',
          a: 'Porque el instalador todavía no tiene una firma digital comercial, algo habitual en proyectos de código abierto nuevos. Podés verificar que cada instalador se compiló a partir del código publicado en GitHub.',
        },
        {
          q: '¿Funciona por Escritorio remoto?',
          a: 'NanoPrint imprime en las impresoras instaladas en la computadora donde corre. Las impresoras redirigidas por Escritorio remoto con «Easy Print» no pueden recibir tickets: instalá el driver de la impresora en la computadora remota y desactivá Easy Print.',
        },
        {
          q: '¿Puedo imprimir desde un celular o una tablet?',
          a: 'Todavía no. NanoPrint se instala en computadoras con Windows, macOS o Linux y solo acepta conexiones de esa misma computadora. Imprimir desde otros dispositivos de la red local, con tu aprobación para cada uno, está planificado para una próxima versión.',
        },
      ],
    },
  },
  developers: {
    title: 'NanoPrint para desarrolladores: imprimir tickets ESC/POS desde JavaScript',
    description:
      'Cómo imprimir tickets en impresoras térmicas desde una página web con NanoPrint y qz-tray.js: librerías compatibles, ejemplo de código y una demo en vivo.',
    hero: {
      eyebrow: 'Para desarrolladores',
      title: 'Imprimí tickets ESC/POS desde JavaScript',
      lead: 'NanoPrint no tiene un SDK propio: tu página usa qz-tray.js, el cliente de QZ Tray, y envía bytes ESC/POS armados con cualquier encoder. Si tu sistema ya imprime con qz-tray.js, no hace falta cambiar nada.',
      guide: 'Guía de integración',
      protocol: 'Protocolo',
    },
    libraries: {
      title: 'Librerías compatibles',
      lead: 'Las que recomendamos, probadas con NanoPrint.',
      license: 'Licencia',
      items: [
        {
          name: 'qz-tray',
          href: 'https://www.npmjs.com/package/qz-tray',
          license: 'LGPL-2.1',
          role: 'Se conecta con NanoPrint y envía los trabajos.',
          note: 'Versión 2.2 o posterior. Cada versión de NanoPrint se prueba con qz-tray.js 2.3.0.',
        },
        {
          name: '@point-of-sale/receipt-printer-encoder',
          href: 'https://www.npmjs.com/package/@point-of-sale/receipt-printer-encoder',
          license: 'MIT',
          role: 'Arma los bytes ESC/POS: texto, páginas de códigos, QR, imágenes y corte.',
          note: 'Usala junto con qz-tray.',
        },
        {
          name: 'web-escpos-printer',
          href: 'https://www.npmjs.com/package/web-escpos-printer',
          license: 'MIT',
          role: 'Encoder y transportes en un solo paquete.',
          note: 'Usá su transporte de QZ Tray, instalado desde npm.',
        },
      ],
    },
    example: {
      title: 'Ejemplo mínimo',
      lead: 'Instalá las dos librerías con `npm install qz-tray @point-of-sale/receipt-printer-encoder` y enviá un ticket:',
      code: `import qz from 'qz-tray';
import ReceiptPrinterEncoder from '@point-of-sale/receipt-printer-encoder';

await qz.websocket.connect();
const printer = await qz.printers.getDefault(); // o qz.printers.find('POS-58')

const bytes = new ReceiptPrinterEncoder({ columns: 32 }) // 58 mm; 48 para 80 mm
  .initialize()
  .line('Kiosco Don Pepe')
  .line('Total: $ 1.234,00')
  .qrcode('https://example.com/ticket/123')
  .cut()
  .encode();

const config = qz.configs.create(printer, { jobName: 'Ticket 123' });
await qz.print(config, [
  { type: 'raw', format: 'command', flavor: 'base64', data: toBase64(bytes) },
]);

function toBase64(bytes) {
  let binary = '';
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary);
}`,
      note: 'Si la impresora predeterminada del sistema es virtual (por ejemplo, «Microsoft Print to PDF»), `getDefault()` devuelve `null`: buscala por nombre o dejá que el usuario la elija.',
    },
    identity: {
      title: 'Nombre y logo de tu sitio',
      text: 'Opcional: NanoPrint muestra en el pedido de permiso el nombre y el logo que envíe tu página, siempre junto a su dirección real. El logo va como data URL PNG, JPEG o WebP de hasta 32 KiB.',
      code: `// Antes de conectar
qz.security.setCertificatePromise((resolve) => {
  resolve(JSON.stringify({
    nanoprint: { name: 'Kiosco Don Pepe', icon: 'data:image/png;base64,…' },
  }));
});`,
    },
    demo: {
      title: 'Probalo ahora',
      lead: 'Con NanoPrint instalado y abierto, esta página se conecta, lista tus impresoras e imprime un ticket de ejemplo. Usa solo qz-tray.js y ReceiptPrinterEncoder.',
      steps: [
        'NanoPrint te va a preguntar si este sitio puede imprimir.',
        'Chrome puede pedir permiso para acceder a dispositivos de la red local: aceptalo para que la página llegue a NanoPrint.',
      ],
      connect: 'Conectar con NanoPrint',
      connecting: 'Conectando…',
      connected: 'Conectado a NanoPrint',
      disconnect: 'Desconectar',
      printer: 'Impresora',
      paper: 'Papel',
      qr: 'Incluir código QR',
      print: 'Imprimir ticket de ejemplo',
      printing: 'Imprimiendo…',
      printed: 'Ticket enviado a {printer}.',
      noPrinters:
        'NanoPrint no le ofrece ninguna impresora a este sitio. Revisá la pestaña Impresoras de NanoPrint.',
      notRunning:
        'No pudimos conectar con NanoPrint. Revisá que esté instalado y abierto (ícono junto al reloj) y que los sitios seguros estén activados en Preferencias.',
      blocked:
        'NanoPrint no permitió la conexión. Si bloqueaste este sitio, desbloquealo en la pestaña Sitios de NanoPrint.',
      failed: 'No se pudo imprimir: {message}',
      download: 'Descargar NanoPrint',
      ticket: {
        title: 'Ticket de ejemplo',
        items: [
          ['Café con leche', 1, 2500],
          ['Medialunas', 3, 900],
          ['Agua mineral', 2, 1200],
        ],
        total: 'TOTAL',
        thanks: '¡Gracias por probar NanoPrint!',
        jobName: 'Ticket de ejemplo',
      },
    },
    requirements: {
      title: 'Antes de integrar',
      items: [
        'Páginas https: el usuario activa una sola vez los «sitios seguros» en NanoPrint, que se lo ofrece al instalar.',
        'Chrome puede pedir permiso de acceso a la red local antes de conectarse con NanoPrint.',
        'Puertos: los mismos que qz-tray.js (`wss://localhost:8181` y `ws://localhost:8182`). Si el usuario elige otros, pasalos en `qz.websocket.connect({ port: … })`.',
        'Límites: 30 trabajos por minuto por sitio, hasta 10 copias por trabajo y mensajes de hasta 2 MiB.',
        'Solo impresión directa: archivos, puertos serie, USB, HID e impresión de píxeles, HTML o PDF no están disponibles.',
      ],
    },
    more: 'Documentación completa en GitHub',
  },
  footer: {
    productOf: 'NanoPrint es un producto de',
    links: {
      github: 'Código fuente',
      license: 'Licencia GPL-3.0',
      security: 'Seguridad',
      licenses: 'Licencias de terceros',
      developers: 'Desarrolladores',
    },
    legal: [
      'NanoPrint se distribuye sin ninguna garantía, según los términos de la licencia GPL-3.0.',
      'QZ Tray es una marca de QZ Industries, LLC. NanoPrint es un proyecto independiente, no afiliado ni respaldado por QZ Industries; lo menciona solo para describir la compatibilidad.',
      'Windows es una marca de Microsoft Corporation. macOS es una marca de Apple Inc. Linux es una marca registrada de Linus Torvalds. Las demás marcas pertenecen a sus dueños.',
    ],
    privacy: 'Este sitio no usa cookies ni herramientas de seguimiento.',
  },
  code: { copy: 'Copiar', copied: 'Copiado' },
};
