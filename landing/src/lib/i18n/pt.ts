import type { Messages } from './types';

// Portuguese as written in Brazil, like the app; it also serves the rest of the Portuguese-speaking
// world. Interface names (Windows, macOS, NanoPrint's tabs) follow their Brazilian Portuguese
// versions. Text between backticks renders as code.
export const pt: Messages = {
  meta: { locale: 'pt_BR' },
  nav: {
    features: 'Recursos',
    download: 'Baixar',
    developers: 'Desenvolvedores',
    faq: 'Dúvidas',
    github: 'GitHub',
    language: 'Idioma',
    languageName: 'Português',
    skip: 'Pular para o conteúdo',
    menu: 'Menu',
  },
  download: {
    generic: 'Baixar o NanoPrint',
    windows: 'Baixar para Windows',
    mac: 'Baixar para macOS',
    linux: 'Baixar para Linux',
    linuxDetail: 'Ubuntu, Debian, Fedora, Raspberry Pi OS e outras distribuições',
    otherPlatforms: 'Outros sistemas',
    previousVersions: 'Versões anteriores',
    desktopOnly: 'O NanoPrint funciona em computadores com Windows, macOS e Linux.',
    starting: 'Seu download está começando…',
    nextSteps: 'Quando o download terminar:',
    retry: 'Não começou? Baixe de novo',
  },
  home: {
    title: 'NanoPrint: imprima cupons e comandas da web na sua impressora térmica',
    description:
      'Aplicativo gratuito e de código aberto para Windows, macOS e Linux que imprime cupons, recibos e comandas do seu sistema web em impressoras térmicas ESC/POS. Cada site pede permissão uma vez, não a cada cupom.',
    hero: {
      title: 'Imprima cupons da web direto na sua impressora térmica',
      lead: 'O NanoPrint é um aplicativo leve e gratuito que conecta o seu sistema web às impressoras de cupom do computador. Cada site pede permissão uma vez, não a cada cupom.',
      platforms: 'Windows · macOS · Linux · Raspberry Pi',
      free: 'Gratuito',
      openSource: 'Código aberto',
      videoLabel:
        'Reproduzir a animação de novo: uma impressora térmica imprime um cupom do NanoPrint.',
    },
    features: {
      title: 'Feito para o balcão',
      lead: 'O que uma loja ou uma cozinha precisa: cupons e comandas que saem rápido, sem configurações complicadas e sem riscos.',
      items: [
        {
          title: 'Multiplataforma',
          text: 'O mesmo sistema web imprime do mesmo jeito no Windows, no macOS e no Linux, com as impressoras que você já tem instaladas.',
          points: [
            'Windows 10 e 11',
            'macOS 10.15 ou posterior, Intel e Apple silicon',
            'Ubuntu, Debian, Fedora e outras distribuições',
            'Raspberry Pi com Raspberry Pi OS de 64 bits',
          ],
        },
        {
          title: 'Leve e gratuito',
          text: 'Um instalador de poucos megabytes, gratuito e sem cadastro. Ele fica na área de notificação e inicia com o computador.',
          points: [
            'Sem custo, assinaturas nem limites de uso',
            'Não precisa de Java nem de outros programas',
            'No Windows, instala sem permissões de administrador',
          ],
        },
        {
          title: 'Seguro, sem incomodar',
          text: 'Nenhum site imprime sem a sua permissão. Você a concede uma vez por site, e ela não é pedida de novo a cada cupom.',
          points: [
            'Permissão por site, revogável a qualquer momento',
            'Aceita conexões apenas deste computador',
            'Nunca armazena o que é impresso',
          ],
        },
      ],
    },
    how: {
      title: 'Como funciona',
      items: [
        {
          title: 'Instale o NanoPrint',
          text: 'Ele fica na área de notificação, perto do relógio, e inicia com o computador.',
        },
        {
          title: 'Abra o seu sistema web',
          text: 'Na primeira vez que um site quiser imprimir, o NanoPrint pergunta. Você responde uma vez: ele não pergunta a cada cupom.',
        },
        {
          title: 'Imprima',
          text: 'O cupom ou a comanda vai direto para a impressora, sem a janela de impressão do navegador.',
        },
      ],
    },
    integrate: {
      title: 'O seu sistema web imprime cupons ou comandas?',
      text: 'Ofereça o NanoPrint aos seus usuários: eles instalam de graça, autorizam o seu site uma vez e os seus cupons vão direto para a impressora deles.',
      points: [
        'Poucas linhas de JavaScript, com bibliotecas conhecidas',
        'Se o seu sistema já usa qz-tray.js, funciona sem mudanças',
        'Gratuito para você e para os seus usuários, sem licenças por estação',
        'O seu sistema pode ter qualquer licença: o NanoPrint é um programa separado',
      ],
      button: 'Guia para desenvolvedores',
      code: `import qz from 'qz-tray';

await qz.websocket.connect();
const config = qz.configs.create('POS-58');
await qz.print(config, [{ type: 'raw', format: 'command', flavor: 'base64', data }]);`,
    },
    printers: {
      title: 'Impressoras compatíveis',
      text: 'Impressoras térmicas de cupom e de cozinha que entendem ESC/POS, de 58 ou 80 mm, conectadas por USB ou pela rede. ESC/POS é a linguagem da maioria das impressoras de cupom.',
      driversTitle: 'Preciso de um driver?',
      drivers: [
        {
          os: 'Windows',
          text: 'A impressora precisa aparecer em Configurações → Impressoras e scanners. Funciona com o driver do fabricante e também com o driver “Genérico / Somente Texto” que vem com o Windows: o NanoPrint envia os comandos como estão.',
        },
        {
          os: 'macOS',
          text: 'Adicione a impressora em Ajustes do Sistema → Impressoras e Scanners, com qualquer driver. O NanoPrint envia dados brutos, então o driver não os altera.',
        },
        {
          os: 'Linux',
          text: 'Adicione a impressora nas configurações de impressão (CUPS), com o driver do fabricante ou como impressora genérica. O NanoPrint imprime sem filtros, em modo raw.',
        },
        {
          os: 'Impressora de rede',
          text: 'Não precisa de driver. Adicione a impressora na aba Impressoras do NanoPrint pelo endereço IP e pela porta (geralmente 9100) e configure um IP fixo para ela no roteador, para que ele não mude. Ela também pode ser instalada no sistema como qualquer outra impressora.',
        },
      ],
      note: 'O NanoPrint não instala drivers, por isso não precisa de permissões de administrador. Impressoras virtuais (PDF, XPS, fax) não são oferecidas aos sites.',
    },
    downloads: {
      title: 'Baixe o NanoPrint',
      lead: 'Gratuito, sem cadastro. Escolha o seu sistema.',
      requirements: 'Requisitos',
      install: 'Instalação',
      windows: {
        name: 'Windows',
        requirements: 'Windows 10 ou 11, 64 bits',
        files: [{ label: 'Instalador (.exe)', download: 'windows' }],
        steps: [
          'Abra o arquivo baixado.',
          'Se aparecer “O Windows protegeu o computador”, clique em “Mais informações” e depois em “Executar assim mesmo”. O aviso aparece porque o instalador ainda não tem uma assinatura digital comercial.',
          'Siga o assistente. Ele não pede permissões de administrador.',
        ],
      },
      mac: {
        name: 'macOS',
        requirements: 'macOS 10.15 ou posterior, Intel e Apple silicon',
        files: [{ label: 'Imagem de disco (.dmg)', download: 'mac' }],
        steps: [
          'Abra o arquivo .dmg e arraste o NanoPrint para Aplicativos.',
          'Na primeira vez, o macOS avisa que não é possível verificar o desenvolvedor. Abra Ajustes do Sistema → Privacidade e Segurança e clique em “Abrir Mesmo Assim”.',
          'Confirme com a sua senha. A partir daí, ele abre direto.',
        ],
      },
      linux: {
        name: 'Linux',
        requirements: 'Intel ou AMD de 64 bits: Ubuntu 22.04, Debian 12, Fedora 38 ou posterior',
        files: [
          { label: '.deb (Ubuntu, Debian)', download: 'deb' },
          { label: '.rpm (Fedora, openSUSE)', download: 'rpm' },
          { label: 'AppImage (outras)', download: 'appImage' },
        ],
        arm: {
          title: 'ARM64: Raspberry Pi OS de 64 bits',
          files: [
            { label: '.deb (Raspberry Pi OS)', download: 'debArm' },
            { label: 'AppImage (outras)', download: 'appImageArm' },
          ],
        },
        steps: [
          'Ubuntu ou Debian: `sudo apt install ./NanoPrint_amd64.deb`',
          'Fedora: `sudo dnf install ./NanoPrint_x86_64.rpm`',
          'Raspberry Pi OS: `sudo apt install ./NanoPrint_arm64.deb`',
          'AppImage: `chmod +x NanoPrint_amd64.AppImage` (no ARM64, `NanoPrint_aarch64.AppImage`) e depois abra o arquivo.',
        ],
      },
      afterTitle: 'Depois de instalar',
      afterText:
        'O NanoPrint dá as boas-vindas e oferece ativar os sites seguros (https) com um clique. Depois, ele fica na área de notificação, perto do relógio.',
      previous: 'Versões anteriores e as novidades de cada uma',
      verifyTitle: 'Verifique o instalador',
      verifyText:
        'Cada instalador é gerado no GitHub a partir do código publicado e traz um atestado de procedência. A ferramenta `gh` do GitHub faz a verificação: `gh attestation verify NanoPrint_x64-setup.exe --repo nanofactura/nanoprint`',
    },
    signing: {
      title: 'Política de assinatura de código',
      items: [
        'Os instaladores para Windows vão ser assinados pelo programa da SignPath Foundation para projetos de código aberto. A solicitação está em andamento: as versões atuais ainda não são assinadas. “Free code signing provided by SignPath.io, certificate by SignPath Foundation”.',
        'Cada instalador é gerado pelo GitHub Actions a partir do código publicado, e um mantenedor do projeto aprova cada assinatura.',
        'Autor, revisor e aprovador: Mariano Paulin (@marianopaulin02).',
        'Privacidade: o NanoPrint não transfere nenhuma informação para outros sistemas em rede, a menos que a pessoa que o usa peça isso expressamente. A única exceção, desde a versão 0.2.0, é a consulta ao GitHub em busca de novas versões, um minuto depois de iniciar e a cada seis horas.',
      ],
    },
    openSource: {
      title: 'Código aberto',
      text: 'O NanoPrint é software livre sob a licença GPL-3.0. Todo o código está no GitHub: revise, relate problemas ou contribua.',
      button: 'Ver no GitHub',
      alternativeTo: 'NanoPrint no AlternativeTo',
    },
    faq: {
      title: 'Perguntas frequentes',
      items: [
        {
          q: 'O NanoPrint é gratuito?',
          a: 'Sim, totalmente: para uso pessoal e comercial, sem cadastro, assinaturas nem limites. É software livre sob a licença GPL-3.0.',
        },
        {
          q: 'Ele vai pedir permissão toda vez que eu imprimir?',
          a: 'Não. O NanoPrint pergunta uma vez por site. Para sites seguros (https), a resposta fica guardada; para sites sem cadeado (http), ele pergunta de novo quando o NanoPrint reinicia, porque outro computador da rede poderia se passar por eles.',
        },
        {
          q: 'Quais impressoras funcionam?',
          a: 'Impressoras térmicas ESC/POS de cupom e de cozinha, de 58 ou 80 mm, conectadas por USB ou pela rede: instaladas no sistema ou, no caso das impressoras de rede, adicionadas no NanoPrint pelo endereço IP. O painel do NanoPrint imprime uma página de teste e define quais impressoras os sites podem ver.',
        },
        {
          q: 'Preciso de um driver especial?',
          a: 'Não. Uma impressora de rede pode ser adicionada no NanoPrint pelo endereço IP, sem driver. Nos outros casos, a impressora precisa estar instalada no sistema operacional, com o driver do fabricante ou um genérico. O NanoPrint envia os comandos como estão.',
        },
        {
          q: 'Uma página da web pode imprimir sem eu perceber?',
          a: 'Não. Todo site precisa pedir permissão, e quem decide é você. O NanoPrint só aceita conexões deste computador, e as permissões podem ser revogadas no painel dele.',
        },
        {
          q: 'Preciso deixar o NanoPrint aberto?',
          a: 'O NanoPrint inicia com o computador e fica na área de notificação, sem atrapalhar. Se você fechá-lo, os sites que o integram podem abri-lo de novo: o seu navegador pergunta antes.',
        },
        {
          q: 'O que o NanoPrint muda no meu computador?',
          a: 'Ele adiciona uma entrada para iniciar com o computador (você pode desativá-la em Preferências) e o link nanoprint:// para que os sites possam abri-lo, sempre depois da sua confirmação. Se você ativar os sites seguros, ele instala um certificado próprio para o seu usuário, depois que o sistema pede a sua confirmação. As configurações e o histórico ficam no seu perfil. O desinstalador do Windows remove tudo quando você escolhe apagar os dados do aplicativo.',
        },
        {
          q: 'O NanoPrint guarda o que eu imprimo?',
          a: 'Não. O histórico guarda a data, o site, a impressora e o resultado de cada trabalho por 30 dias, nunca o conteúdo. O NanoPrint só se conecta ao GitHub para buscar novas versões; com as atualizações automáticas desativadas, ele avisa e você decide quando instalar.',
        },
        {
          q: 'Por que o Windows e o macOS mostram um aviso na instalação?',
          a: 'Porque o instalador ainda não tem uma assinatura digital comercial, algo comum em projetos de código aberto novos. Você pode verificar que cada instalador foi gerado a partir do código publicado no GitHub.',
        },
        {
          q: 'Funciona com sistemas que hoje imprimem pelo QZ Tray?',
          a: 'Normalmente, sim: o NanoPrint entende o protocolo do qz-tray.js para imprimir cupons, então esses sistemas costumam funcionar sem mudanças. Os recursos do QZ Tray além da impressão de cupons não estão incluídos.',
        },
        {
          q: 'Posso integrar o NanoPrint ao meu software comercial?',
          a: 'Sim. O seu software só se comunica com o NanoPrint, que é um programa separado, então pode ter qualquer licença. A página para desenvolvedores explica como fazer a integração.',
        },
        {
          q: 'Funciona pela Área de Trabalho Remota?',
          a: 'O NanoPrint imprime nas impressoras instaladas no computador em que ele roda. As impressoras redirecionadas pela Área de Trabalho Remota com o “Easy Print” não conseguem receber cupons: instale o driver da impressora no computador remoto e desative o Easy Print.',
        },
        {
          q: 'Funciona em um Raspberry Pi?',
          a: 'Sim, desde a versão 0.5.0, no Raspberry Pi OS de 64 bits com área de trabalho: baixe o .deb para ARM64. O sistema web precisa abrir no navegador do próprio Raspberry Pi, porque o NanoPrint só aceita conexões do computador em que está instalado. A impressora se conecta por USB ou pela rede, como em qualquer computador.',
        },
        {
          q: 'Posso imprimir de um celular ou de um tablet?',
          a: 'Ainda não. O NanoPrint é instalado em computadores com Windows, macOS e Linux e só aceita conexões desse mesmo computador. Imprimir de outros dispositivos da rede local, cada um aprovado por você, está nos planos para uma versão futura.',
        },
      ],
    },
  },
  developers: {
    title: 'NanoPrint para desenvolvedores: imprima cupons e comandas ESC/POS com JavaScript',
    description:
      'Como imprimir cupons e comandas em impressoras térmicas a partir do seu sistema web com o NanoPrint: bibliotecas JavaScript compatíveis, um exemplo de código e uma demonstração ao vivo.',
    hero: {
      eyebrow: 'Para desenvolvedores',
      title: 'Imprima cupons ESC/POS com JavaScript',
      lead: 'A sua página se conecta ao NanoPrint pelo qz-tray.js e envia cupons ESC/POS gerados com o codificador que você preferir. Se o seu sistema já usa qz-tray.js, funciona sem mudanças.',
      guide: 'Guia de integração',
      protocol: 'Protocolo',
    },
    libraries: {
      title: 'Bibliotecas compatíveis',
      lead: 'As que recomendamos, testadas com o NanoPrint.',
      license: 'Licença',
      items: [
        {
          name: 'qz-tray',
          href: 'https://www.npmjs.com/package/qz-tray',
          license: 'LGPL-2.1',
          role: 'Conecta-se ao NanoPrint e envia os trabalhos.',
          note: 'Versão 2.2 ou posterior. Cada versão do NanoPrint é testada com o qz-tray.js 2.3.0.',
        },
        {
          name: '@point-of-sale/receipt-printer-encoder',
          href: 'https://www.npmjs.com/package/@point-of-sale/receipt-printer-encoder',
          license: 'MIT',
          role: 'Gera os bytes ESC/POS: texto, páginas de código, códigos QR, imagens e corte.',
          note: 'Use junto com o qz-tray.',
        },
        {
          name: 'web-escpos-printer',
          href: 'https://www.npmjs.com/package/web-escpos-printer',
          license: 'MIT',
          role: 'Codificador e transportes em um só pacote.',
          note: 'Use o transporte para QZ Tray dele, instalado pelo npm.',
        },
      ],
    },
    example: {
      title: 'Exemplo mínimo',
      lead: 'Instale as duas bibliotecas com `npm install qz-tray @point-of-sale/receipt-printer-encoder` e envie um cupom:',
      code: `import qz from 'qz-tray';
import ReceiptPrinterEncoder from '@point-of-sale/receipt-printer-encoder';

await qz.websocket.connect();
const printer = await qz.printers.getDefault(); // ou qz.printers.find('POS-58')

const bytes = new ReceiptPrinterEncoder({ columns: 32 }) // 58 mm; 48 para 80 mm
  .initialize()
  .line('Lanchonete do Zé')
  .line('Total: R$ 45,90')
  .qrcode('https://example.com/cupom/123')
  .cut()
  .encode();

const config = qz.configs.create(printer, { jobName: 'Cupom 123' });
await qz.print(config, [
  { type: 'raw', format: 'command', flavor: 'base64', data: toBase64(bytes) },
]);

function toBase64(bytes) {
  let binary = '';
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary);
}`,
      note: 'Se a impressora padrão do sistema for virtual (como a “Microsoft Print to PDF”), `getDefault()` retorna `null`: procure a impressora pelo nome ou deixe o usuário escolher.',
    },
    identity: {
      title: 'O nome e o logo do seu site',
      text: 'Opcional: o NanoPrint mostra no pedido de permissão o nome e o logo que a sua página envia, sempre ao lado do endereço real da página. O logo é uma data URL PNG, JPEG ou WebP de até 32 KiB.',
      code: `// Antes de conectar
qz.security.setCertificatePromise((resolve) => {
  resolve(JSON.stringify({
    nanoprint: { name: 'Lanchonete do Zé', icon: 'data:image/png;base64,…' },
  }));
});`,
    },
    wake: {
      title: 'Quando o NanoPrint está fechado',
      text: 'O NanoPrint inicia com o computador, mas o usuário pode tê-lo fechado. A sua página pode iniciá-lo de novo com o link `nanoprint://start`: o navegador pergunta antes de abri-lo, e o NanoPrint inicia na área de notificação, sem janela.',
      code: `async function openNanoPrint() {
  window.location.href = 'nanoprint://start';
  // O NanoPrint precisa de um instante para iniciar
  await qz.websocket.connect({ retries: 15, delay: 1 });
}`,
      note: 'Ofereça isso como um botão “Abrir o NanoPrint” depois de uma conexão que falhou: os navegadores só abrem aplicativos depois de um clique. O link só inicia o NanoPrint, não leva dados, e o site continua precisando de permissão para imprimir.',
    },
    demo: {
      title: 'Teste agora',
      lead: 'Com o NanoPrint instalado e em execução, esta página se conecta a ele, lista as suas impressoras e imprime um cupom de exemplo. Ela usa apenas o qz-tray.js e o ReceiptPrinterEncoder.',
      steps: [
        'O NanoPrint vai perguntar se este site pode imprimir.',
        'O Chrome pode pedir para permitir o acesso a dispositivos da sua rede local: permita, para que a página consiga chegar ao NanoPrint.',
      ],
      connect: 'Conectar ao NanoPrint',
      wake: 'Abrir o NanoPrint',
      wakeFailed:
        'O NanoPrint não abriu. Se ele não estiver instalado, baixe-o; se o navegador perguntou e você recusou, tente de novo.',
      connecting: 'Conectando…',
      connected: 'Conectado ao NanoPrint',
      disconnect: 'Desconectar',
      printer: 'Impressora',
      paper: 'Papel',
      qr: 'Incluir um código QR',
      print: 'Imprimir um cupom de exemplo',
      printing: 'Imprimindo…',
      printed: 'Cupom enviado para {printer}.',
      noPrinters:
        'O NanoPrint não oferece nenhuma impressora a este site. Confira a aba Impressoras no painel do NanoPrint.',
      notRunning:
        'Não foi possível conectar ao NanoPrint. Se ele estiver instalado mas fechado, abra-o com o botão (o navegador vai pedir confirmação). Se ele estiver em execução, confira se os sites seguros estão ativados em Preferências.',
      blocked:
        'O NanoPrint recusou a conexão. Se você bloqueou este site, desbloqueie-o na aba Sites do NanoPrint.',
      failed: 'Não foi possível imprimir: {message}',
      download: 'Baixar o NanoPrint',
      siteName: 'Demonstração do NanoPrint',
      ticket: {
        title: 'Cupom de exemplo',
        // The encoder's default code page (CP437) has no ã or Á: these names print as written.
        items: [
          ['Café com leite', 1, 7.5],
          ['Coxinha', 3, 6.5],
          ['Suco de laranja', 2, 8],
        ],
        total: 'TOTAL',
        thanks: 'Obrigado por testar o NanoPrint!',
        jobName: 'Cupom de exemplo',
      },
    },
    adopt: {
      title: 'Ofereça aos seus usuários',
      lead: 'O NanoPrint foi feito para ser usado por outros projetos.',
      items: [
        'Para o download, use um link para `nanoprint.nanofactura.com/pt/`: a página detecta o sistema de cada usuário e o guia na instalação.',
        'É gratuito para você e para os seus usuários: sem licenças por estação, assinaturas nem limites de uso.',
        'O seu sistema só se comunica com o NanoPrint, um programa separado, então pode ter qualquer licença, inclusive proprietária.',
        'Se faltar algo ou algo não funcionar, conte para nós no GitHub: integrações reais nos ajudam a decidir o que vem depois.',
      ],
      issues: 'Abrir uma issue no GitHub',
    },
    requirements: {
      title: 'Antes de integrar',
      items: [
        'Páginas https: o usuário ativa os “sites seguros” no NanoPrint uma vez; o NanoPrint oferece isso logo depois da instalação.',
        'O Chrome pode pedir acesso à rede local antes que a página consiga se conectar ao NanoPrint.',
        'Portas: as mesmas do qz-tray.js (`wss://localhost:8181` e `ws://localhost:8182`). Se o usuário escolher outras, informe-as em `qz.websocket.connect({ port: … })`.',
        'Limites: 30 trabalhos por minuto por site, até 10 cópias por trabalho e mensagens de até 2 MiB.',
        'Somente impressão raw: arquivos, portas seriais, USB, HID e impressão em pixels, HTML ou PDF não estão disponíveis.',
      ],
    },
    more: 'Documentação completa no GitHub',
  },
  footer: {
    productOf: 'O NanoPrint é um produto da',
    links: {
      github: 'Código-fonte',
      license: 'Licença GPL-3.0',
      security: 'Segurança',
      licenses: 'Licenças de terceiros',
      developers: 'Desenvolvedores',
      signing: 'Política de assinatura de código',
    },
    legal: [
      'O NanoPrint não oferece absolutamente nenhuma garantia, nos termos da licença GPL-3.0.',
      'QZ Tray é uma marca da QZ Industries, LLC. O NanoPrint é um projeto independente, não afiliado à QZ Industries nem endossado por ela; ele menciona o QZ Tray apenas para descrever a compatibilidade.',
      'Windows é uma marca da Microsoft Corporation. macOS é uma marca da Apple Inc. Linux é uma marca registrada de Linus Torvalds. Raspberry Pi é uma marca da Raspberry Pi Ltd. As demais marcas pertencem aos seus proprietários.',
    ],
    privacy: 'Este site não usa cookies nem rastreamento.',
  },
  code: { copy: 'Copiar', copied: 'Copiado' },
};
