# Documento de Requisitos - Personalización Visual WebUI (Anáhuac Mayab)

**Proyecto**: ActivityWatch WebUI - Branding Anáhuac Mayab  
**Fecha**: 2026-05-19  
**Tipo**: UI/UX Enhancement (Brownfield)  
**Fase**: INCEPTION - Requirements Analysis  
**Versión**: 1.0

---

## 1. Resumen Ejecutivo

### 1.1 Solicitud Original
**Usuario**: "Modificar el diseño visual de AW-webui, cambiar el logo.png y personalizar la pantalla de inicio. No modifiques ningun otro componente y manten todas las consultas como estan diseñadas"

### 1.2 Tipo de Solicitud
- **Categoría**: UI/UX Enhancement
- **Complejidad**: Moderada (cambios visuales concentrados, no funcionales)
- **Riesgo**: Bajo (solo cambios estéticos, sin impacto en backend o queries)
- **Alcance**: Limitado y bien definido (logo + pantalla de inicio)

### 1.3 Objetivos Principales
1. ✅ Aplicar branding institucional de Anáhuac Mayab al WebUI
2. ✅ Reemplazar logo actual con logo de Anáhuac
3. ✅ Rediseñar completamente la pantalla de inicio
4. ✅ Mantener 100% de funcionalidad existente sin cambios
5. ✅ Preservar todas las queries tal como están diseñadas
6. ✅ Mantener compatibilidad con diseño actual de ActivityWatch

---

## 2. Análisis del Request

### 2.1 Claridad del Request
**Evaluación**: ⭐⭐⭐⭐⭐ Excelente claridad
- Request específico y bien definido
- Alcance claramente delimitado (solo visual)
- Restricciones explícitas (no queries, no otros componentes)
- Especificaciones de diseño completas proporcionadas

### 2.2 Tipo de Request
**Tipo Principal**: Enhancement (Mejora visual/branding)  
**Sub-tipos**:
- Rebranding corporativo
- Personalización de UI
- Asset replacement (logo)
- Layout redesign (pantalla inicio)

### 2.3 Alcance Inicial
**Alcance**: Single Component (aw-webui únicamente)  
**Componentes Afectados**: 1 de 3 (solo aw-webui, no aw-server ni postgresql)  
**Archivos Estimados**: 5-8 archivos en aw-webui

### 2.4 Complejidad Inicial
**Complejidad**: Moderada  
**Justificación**:
- ✅ Simple: Solo cambios visuales (CSS, assets, componentes Vue)
- ⚠️ Moderado: Requiere clonar repo externo y modificar Dockerfile
- ✅ Sin complejidad técnica: No hay lógica de negocio ni integraciones

---

## 3. Requisitos Funcionales

### RF-1: Reemplazo de Logo

**Prioridad**: Alta  
**Descripción**: Reemplazar el logo actual de ActivityWatch con el isotipo de Anáhuac Mayab

**Detalles**:
- **Archivo fuente**: `assets/Anáhuac_Isotipo_RGB_Negro_Positivo.jpg`
- **Formato destino**: Convertir JPG → PNG con transparencia (o SVG)
- **Ubicaciones**: 
  - Header/navbar (todas las páginas)
  - Pantalla de inicio (tamaño más grande)
- **Dimensiones**: Mantener dimensiones actuales del logo original de ActivityWatch
- **Border radius**: 12px (var(--border-radius-lg))

**Criterios de Aceptación**:
- [ ] Logo de Anáhuac visible en header de todas las páginas
- [ ] Logo de Anáhuac visible en pantalla de inicio (más grande)
- [ ] Border radius de 12px aplicado correctamente
- [ ] No hay distorsión ni pixelación del logo
- [ ] Logo mantiene proporciones correctas

**Archivos a Modificar** (estimado):
- `aw-webui/src/assets/logo.png` (o similar)
- `aw-webui/src/components/Header.vue` (referencia al logo)
- `aw-webui/src/views/Home.vue` (logo en pantalla inicio)

---

### RF-2: Rediseño de Pantalla de Inicio

**Prioridad**: Alta  
**Descripción**: Rediseño completo de la pantalla de inicio con branding de Anáhuac Mayab

**Detalles**:
- **Tipo de cambios**: Rediseño completo (texto, layout, contenido, colores, CSS)
- **Contenido**: Usuario tiene idea general, necesita ayuda con texto final
- **Elementos**: Solo texto e información básica (sin imágenes adicionales, sin enlaces externos)
- **Compatibilidad**: Mantener integración con diseño actual de ActivityWatch

**Propuesta de Contenido** (para aprobación del usuario):

**✅ Contenido APROBADO por el Usuario**:
```
[Logo Grande Centrado - 80px height]

ActivityWatch - Anáhuac Mayab
Sistema de Monitoreo de Uso de Software

Bienvenido al sistema de seguimiento de software
y productividad de la Universidad Anáhuac Mayab

[Botón: Comenzar a Monitorear]
```

---

**Opciones Alternativas Consideradas**:

**Opción A (Simple y Directa)** - Recomendada:
```
[Logo Grande Centrado - 80px height]

ActivityWatch - Anáhuac Mayab
Sistema de Monitoreo de Actividades

Bienvenido al sistema de seguimiento de tiempo
y productividad de la Universidad Anáhuac Mayab

[Botón: Comenzar a Monitorear]
```

**Opción B (Más Informativa)**:
```
[Logo Grande Centrado - 80px height]

ActivityWatch
Universidad Anáhuac Mayab

Monitorea tu tiempo de forma automática
y mejora tu productividad

Este sistema registra automáticamente el tiempo que pasas
en diferentes aplicaciones y sitios web, ayudándote a
entender mejor cómo utilizas tu tiempo.

[Botón: Ver Dashboard]
```

**Opción C (Minimalista)**:
```
[Logo Grande Centrado - 80px height]

ActivityWatch
Anáhuac Mayab

Monitoreo de Actividades

[Botón: Comenzar]
```

**Layout Visual Propuesto**:
```
+------------------------------------------+
|  [Header con Logo pequeño]               |
+------------------------------------------+
|                                          |
|            [Espacio 40px]                |
|                                          |
|         [Logo Grande - 80px]             |
|                                          |
|       ActivityWatch - Anáhuac Mayab      |
|       (1.75rem, bold, #040404)           |
|                                          |
|   Sistema de Monitoreo de Uso de Software|
|         (1rem, regular, #9C9C9C)         |
|                                          |
|            [Espacio 24px]                |
|                                          |
|  Bienvenido al sistema de seguimiento de |
|  software y productividad de Anáhuac     |
|         (1rem, regular, #262626)         |
|                                          |
|            [Espacio 32px]                |
|                                          |
|       [Botón: Comenzar a Monitorear]     |
|     (Naranja #FF5900, border-radius      |
|          8px, hover #E04F00)             |
|                                          |
|            [Espacio 40px]                |
|                                          |
+------------------------------------------+
```

**Criterios de Aceptación**:
- [ ] Pantalla de inicio muestra logo grande de Anáhuac (80px)
- [ ] Texto de bienvenida claro y profesional
- [ ] Layout centrado y visualmente balanceado
- [ ] Colores aplicados según paleta de Anáhuac (Naranja #FF5900)
- [ ] Botón CTA funcional con hover effect
- [ ] Responsive en desktop y móvil
- [ ] Usuario aprueba el contenido textual final

**Archivos a Modificar** (estimado):
- `aw-webui/src/views/Home.vue` (o equivalente)
- `aw-webui/src/style/home.scss` (o similar)
- `aw-webui/src/components/HomeHero.vue` (si existe)

---

### RF-3: Aplicación de Paleta de Colores

**Prioridad**: Alta  
**Descripción**: Aplicar la paleta de colores institucional de Anáhuac Mayab globalmente

**Detalles**:
- **Color primario**: Naranja Anáhuac (#FF5900)
- **Colores totales**: 15 colores definidos (ver design-specifications.md)
- **Alcance**: Componentes globales (botones, links, borders)
- **Restricción**: Solo aplicar a elementos visibles en logo + pantalla inicio

**Especificación Completa**: Ver [design-specifications.md](design-specifications.md) Sección 2

**Criterios de Aceptación**:
- [ ] Variables CSS creadas con todos los colores
- [ ] Color primario (#FF5900) aplicado a botones
- [ ] Hover effects con color primario-hover (#E04F00)
- [ ] Textos usan colores definidos (primary, secondary, muted)
- [ ] Consistencia visual en toda la pantalla de inicio

**Archivos a Modificar** (estimado):
- `aw-webui/src/style/variables.scss` (o equivalent)
- `aw-webui/src/style/style.scss` (importar variables)
- `aw-webui/src/style/components/buttons.scss`

---

### RF-4: Implementación de Tipografía

**Prioridad**: Media  
**Descripción**: Aplicar tipografía corporativa Inter/Segoe UI según especificaciones

**Detalles**:
- **Font Family**: Inter (Google Fonts) → Segoe UI → sans-serif
- **Pesos**: 400 (Regular), 500 (Medium), 700 (Bold)
- **Tamaños**: 6 especificaciones por elemento (ver design-specifications.md)

**Especificación Completa**: Ver [design-specifications.md](design-specifications.md) Sección 3

**Criterios de Aceptación**:
- [ ] Inter importada desde Google Fonts (o localmente)
- [ ] Font-family aplicado globalmente
- [ ] Tamaños y pesos correctos en cada elemento
- [ ] Fallback a Segoe UI funciona correctamente

**Archivos a Modificar** (estimado):
- `aw-webui/public/index.html` (importar fuente)
- `aw-webui/src/style/typography.scss` (definiciones)

---

### RF-5: Border Radius Consistente

**Prioridad**: Baja  
**Descripción**: Aplicar border radius según especificaciones (4px, 8px, 12px)

**Detalles**:
- **Small (4px)**: Inputs, badges
- **Medium (8px)**: Botones, cards pequeñas
- **Large (12px)**: Cards, login container, logo

**Especificación Completa**: Ver [design-specifications.md](design-specifications.md) Sección 4

**Criterios de Aceptación**:
- [ ] Variables CSS creadas para 3 tamaños
- [ ] Border radius aplicado a logo (12px)
- [ ] Border radius aplicado a botón principal (8px)
- [ ] Consistencia visual en todos los elementos

**Archivos a Modificar** (estimado):
- `aw-webui/src/style/variables.scss` (definir variables)

---

## 4. Requisitos No Funcionales

### NFR-1: Compatibilidad Visual

**Descripción**: Los cambios deben integrarse bien con el diseño actual de ActivityWatch

**Especificaciones**:
- Mantener estructura de navegación existente
- Preservar layout de otras páginas (solo modificar home)
- Respetar componentes UI existentes (no romper otros views)
- Transiciones suaves entre páginas

**Criterios de Aceptación**:
- [ ] Navegación funciona sin errores
- [ ] Otras páginas (dashboards, settings, etc.) no afectadas
- [ ] No hay inconsistencias visuales entre páginas
- [ ] Usuario puede navegar sin confusión

---

### NFR-2: Funcionalidad Preservada

**Descripción**: Mantener 100% de funcionalidad existente sin cambios

**Especificaciones**:
- ❌ NO modificar queries de ActivityWatch
- ❌ NO modificar componentes backend (aw-server, aw-datastore)
- ❌ NO modificar lógica de negocio
- ✅ Solo cambios estéticos (CSS, assets, HTML/templates)

**Criterios de Aceptación**:
- [ ] Todas las queries funcionan igual que antes
- [ ] API calls sin modificaciones
- [ ] Data processing sin cambios
- [ ] Tests existentes pasan sin errores

---

### NFR-3: Performance

**Descripción**: Los cambios visuales no deben impactar performance

**Especificaciones**:
- Logo optimizado (conversión JPG → PNG/SVG)
- CSS minificado en producción
- Fuentes cargadas de forma optimizada (preconnect, display=swap)
- Imágenes con dimensiones apropiadas

**Criterios de Aceptación**:
- [ ] Tiempo de carga de home page < 2 segundos
- [ ] Logo carga sin delay visible
- [ ] Fuentes cargan con fallback apropiado
- [ ] No hay layout shift durante carga

---

### NFR-4: Responsive Design

**Descripción**: Los cambios deben funcionar en desktop y móvil

**Especificaciones**:
- Desktop: 1920x1080, 1366x768
- Tablet: 768x1024
- Mobile: 375x667, 414x896

**Criterios de Aceptación**:
- [ ] Layout de home page responsive en todas las resoluciones
- [ ] Logo escalable sin distorsión
- [ ] Texto legible en móvil
- [ ] Botones clickeables en touch devices

---

## 5. Alcance de Implementación

### 5.1 In Scope (Incluido)

✅ **Assets**:
- Reemplazar logo de ActivityWatch con Anáhuac Mayab
- Convertir JPG → PNG/SVG
- Optimizar tamaño de archivo

✅ **CSS/Styles**:
- Aplicar paleta de colores de Anáhuac (15 colores)
- Implementar tipografía Inter/Segoe UI
- Definir border radius (3 tamaños)
- Estilos para pantalla de inicio

✅ **Componentes Vue**:
- Modificar Header.vue (logo en navbar)
- Modificar Home.vue (pantalla de inicio completa)
- Actualizar referencias a assets

✅ **HTML/Templates**:
- Importar fuente Inter desde Google Fonts
- Actualizar meta tags si es necesario

✅ **Docker**:
- Modificar Dockerfile.webui para usar código local
- Copiar archivos modificados durante build

✅ **Documentación**:
- Documentar cambios realizados
- Guía de mantenimiento del branding

### 5.2 Out of Scope (Excluido)

❌ **Backend**:
- Sin cambios en aw-server (Rust)
- Sin cambios en aw-datastore
- Sin cambios en aw-query
- Sin cambios en APIs o endpoints

❌ **Funcionalidad**:
- Sin modificar queries de ActivityWatch
- Sin cambios en lógica de negocio
- Sin nuevas features o capacidades
- Sin cambios en data processing

❌ **Otros Componentes UI**:
- Sin cambios en footer (usuario respondió "solo logo + home")
- Sin cambios en otras páginas (dashboard, settings, etc.)
- Sin modificar barra lateral o navegación secundaria
- Sin agregar imágenes/gráficos adicionales

---

## 6. Fases de Implementación

### Fase 1: Preparación (INCEPTION - Current)
- [x] Workspace Detection
- [x] Análisis de requisitos
- [x] Especificaciones de diseño documentadas
- [x] Preguntas de clarificación respondidas
- [ ] **Workflow Planning** (próximo paso)
- [ ] Determinar unidades de trabajo

### Fase 2: Diseño (CONSTRUCTION)
- [ ] Clonar aw-webui repository localmente
- [ ] Análisis de estructura de archivos Vue
- [ ] Mapeo de componentes a modificar
- [ ] Diseño de CSS architecture (variables, imports)
- [ ] Preparación de assets (logo JPG → PNG/SVG)

### Fase 3: Implementación (CONSTRUCTION - Code Generation)
- [ ] Conversión de logo (JPG → PNG con transparencia)
- [ ] Creación de variables CSS (colores, tipografía, border radius)
- [ ] Importación de fuente Inter
- [ ] Modificación de Header.vue (logo pequeño)
- [ ] Rediseño completo de Home.vue (logo grande + contenido)
- [ ] Actualización de Dockerfile.webui (usar código local)
- [ ] Aplicación de estilos globales

### Fase 4: Validación (CONSTRUCTION - Build and Test)
- [ ] Build local de aw-webui (npm run build)
- [ ] Build de Docker image (docker compose build aw-webui)
- [ ] Validación visual (revisión manual)
- [ ] Test de navegación (header logo clickeable)
- [ ] Test responsive (desktop, tablet, mobile)
- [ ] Test en múltiples navegadores
- [ ] Validación de performance (tiempos de carga)

---

## 7. Criterios de Éxito

### 7.1 Criterios Técnicos

✅ **Build y Deployment**:
- [ ] `docker compose build aw-webui` ejecuta sin errores
- [ ] Imagen Docker creada correctamente (< 100 MB ideal)
- [ ] `docker compose up` inicia aw-webui sin errores
- [ ] Healthcheck de aw-webui pasa consistentemente

✅ **Funcionalidad**:
- [ ] Logo de Anáhuac visible en header (todas las páginas)
- [ ] Logo de Anáhuac visible en pantalla de inicio (más grande)
- [ ] Pantalla de inicio muestra contenido rediseñado
- [ ] Botón "Comenzar" (o similar) funciona correctamente
- [ ] Navegación desde home a otras páginas funciona
- [ ] Todas las queries de ActivityWatch funcionan sin cambios

✅ **Visual**:
- [ ] Colores aplicados según paleta de Anáhuac (#FF5900 primario)
- [ ] Tipografía Inter visible en todos los textos
- [ ] Border radius aplicado correctamente (logo 12px, botón 8px)
- [ ] Layout de home page centrado y balanceado
- [ ] Hover effects en botones funcionan

✅ **Performance**:
- [ ] Tiempo de carga de home page < 2 segundos
- [ ] No hay layout shift durante carga
- [ ] Logo carga sin delay visible

✅ **Compatibilidad**:
- [ ] Funciona en Chrome, Firefox, Safari, Edge
- [ ] Responsive en desktop, tablet, mobile

### 7.2 Criterios de Negocio

✅ **Branding**:
- [ ] Logo de Anáhuac Mayab claramente visible
- [ ] Colores institucionales aplicados correctamente
- [ ] Apariencia profesional y consistente con marca Anáhuac
- [ ] Usuario aprueba el resultado visual final

---

## 8. Contenido Aprobado para Pantalla de Inicio ✅

**Usuario seleccionó**: Versión personalizada basada en Opción A

**Contenido Final Aprobado**:
```
[Logo Grande Centrado - 80px height con border-radius 12px]

Título Principal:
"ActivityWatch - Anáhuac Mayab"
(font-size: 1.75rem, font-weight: 700, color: #040404)

Subtítulo:
"Sistema de Monitoreo de Uso de Software"
(font-size: 1rem, font-weight: 400, color: #9C9C9C)

Texto de Bienvenida:
"Bienvenido al sistema de seguimiento de software
y productividad de la Universidad Anáhuac Mayab"
(font-size: 1rem, font-weight: 400, color: #262626)

Call to Action:
[Botón: "Comenzar a Monitorear"]
(background: #FF5900, hover: #E04F00, color: #FFFFFF, 
 font-size: 0.875rem, font-weight: 500, border-radius: 8px,
 padding: 14px 32px)
```

**Aprobado por usuario el**: 2026-05-19T00:30:00Z

---

## 9. Próximos Pasos (Workflow AI-DLC)

Una vez que apruebes este documento y selecciones el contenido de la pantalla de inicio:

1. ✅ **Workflow Planning** - Crear plan de ejecución
2. ✅ **Code Generation** - Implementar cambios
3. ✅ **Build and Test** - Validar con Docker

**Etapas a OMITIR** (por simplicidad del cambio):
- ❌ Reverse Engineering (no necesario)
- ❌ User Stories (cambios visuales simples)
- ❌ Application Design (no hay arquitectura nueva)
- ❌ Functional Design (diseño suficientemente definido aquí)
- ❌ NFR Requirements/Design (ya definidos)
- ❌ Infrastructure Design (Docker ya definido)

---

## 10. Anexos

### Anexo A: Referencias
- [design-specifications.md](design-specifications.md) - Especificaciones completas de diseño
- [webui-design-questions.md](webui-design-questions.md) - Preguntas y respuestas del usuario
- https://forlife.anahuac.mx/colores/ - Guía de colores de Anáhuac Mayab
- https://github.com/ActivityWatch/aw-webui - Repositorio upstream de aw-webui

### Anexo B: Archivos de Logo Disponibles
- `assets/Anáhuac_Isotipo_RGB_Negro_Positivo.jpg` ⭐ (Selección del usuario)
- `assets/Anáhuac_Isotipo_RGB_Blanco_Fondo_Naranja_Positivo.jpg`
- `assets/Anáhuac Mayab_Logotipo_RGB-01.jpg`
- `assets/Anáhuac Mayab_Logotipo_RGB-02.jpg`
- `assets/Anáhuac_Isotipo_RGB_Negro_Negativo.jpg`
- Última actualización**: 2026-05-19T00:30:00Z  
**Estado**: ✅ **APROBADO por el Usuario**  
**Contenido de Home Page**: ✅ **APROBADO** (versión personalizada)  
**Próxima Acción**: Proceder a Workflow Planning
**Documento creado**: 2026-05-19T00:25:00Z  
**Estado**: ⏳ **Pendiente de Aprobación del Usuario**  
**Acción Requerida**: 
1. Aprobar documento de requisitos
2. Seleccionar opción de contenido para pantalla de inicio (A, B, C, u otra)
