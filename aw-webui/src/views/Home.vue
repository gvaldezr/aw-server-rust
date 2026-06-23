<template lang="pug">
div.home-container
  // Hero Section
  div.hero-section
    div.container
      div.row.align-items-center
        div.col-md-12.text-center
          // Logo
          div.logo-container.mb-4
            img.home-logo(src="/logo.png" alt="Anáhuac Mayab" data-testid="home-logo")
          
          // Title
          h1.display-4.font-weight-bold.text-primary.mb-3 ActivityWatch - Anáhuac Mayab
          
          // Subtitle
          h2.h3.text-secondary.mb-4 Sistema de Monitoreo de Uso de Software
          
          // Description
          p.lead.text-muted.mb-5
            | Bienvenido al sistema de seguimiento de software y productividad de la Universidad Anáhuac Mayab
          
          // Call to Action Button
          div.cta-buttons
            router-link.btn.btn-anahuac-primary.btn-lg(to="/activity/0/view" data-testid="home-cta-button")
              | Comenzar a Monitorear
  
  hr.my-5
  
  // Features Section
  div.features-section
    div.container
      div.row
        div.col-md-4.mb-4
          div.feature-card(data-testid="feature-card-1")
            div.feature-icon
              icon(name="chart-line" scale="2")
            h4.mt-3 Monitoreo en Tiempo Real
            p.text-muted Rastrea automáticamente el uso de aplicaciones y sitios web
        
        div.col-md-4.mb-4
          div.feature-card(data-testid="feature-card-2")
            div.feature-icon
              icon(name="user-shield" scale="2")
            h4.mt-3 Privacidad Local
            p.text-muted Todos tus datos permanecen en tu dispositivo
        
        div.col-md-4.mb-4
          div.feature-card(data-testid="feature-card-3")
            div.feature-icon
              icon(name="chart-pie" scale="2")
            h4.mt-3 Reportes Detallados
            p.text-muted Visualiza cómo inviertes tu tiempo con reportes interactivos
  
  hr.my-5
  
  // Resources Section
  div.resources-section
    div.container
      div.row
        div.col-md-6
          h4 Recursos
          ul
            li #[a(href="https://activitywatch.net/" target="_blank") Sitio Web Oficial]
            li #[a(href="https://activitywatch.readthedocs.org/" target="_blank") Documentación]
            li #[a(href="https://forum.activitywatch.net/" target="_blank") Foro de Soporte]
            li #[a(href="https://github.com/ActivityWatch/activitywatch" target="_blank") GitHub]
            li(v-if="!info.version.includes('rust')") #[a(href="/api/" target="_blank") API Browser]
        
        div.col-md-6
          h4 Universidad Anáhuac Mayab
          p
            | Este sistema de monitoreo está personalizado para la comunidad universitaria.
            | Para soporte técnico, contacta al departamento de TI.
          p.small.text-muted
            | Puedes cambiar la página de inicio en #[router-link(to="/settings") configuración].
</template>

<script lang="ts">
import { mapState } from 'pinia';
import { useServerStore } from '~/stores/server';

// Import icons for feature cards
import 'vue-awesome/icons/chart-line';
import 'vue-awesome/icons/user-shield';
import 'vue-awesome/icons/chart-pie';

export default {
  name: 'Home',
  computed: {
    ...mapState(useServerStore, ['info']),
  },
};
</script>

<style lang="scss" scoped>
@import '../style/globals';

.home-container {
  padding: 2rem 0;
}

.hero-section {
  padding: 4rem 0 3rem;
  background: linear-gradient(135deg, $primary-light 0%, $background 100%);
  border-radius: $border-radius-lg;
  margin-bottom: 2rem;
}

.logo-container {
  display: flex;
  justify-content: center;
  align-items: center;
  margin-bottom: 2rem;
}

.home-logo {
  height: 120px;
  width: auto;
  filter: drop-shadow(0 4px 6px rgba(0, 0, 0, 0.1));
  border-radius: $border-radius-lg;
  padding: 1rem;
  background-color: $surface;
}

.text-primary {
  color: $primary-color !important;
}

.text-secondary {
  color: $text-secondary !important;
}

.text-muted {
  color: $text-muted !important;
}

.cta-buttons {
  margin-top: 2rem;
}

.features-section {
  padding: 2rem 0;
}

.feature-card {
  text-align: center;
  padding: 2rem 1rem;
  border-radius: $border-radius-lg;
  background-color: $surface;
  border: 1px solid $border;
  transition: all 0.3s ease;
  height: 100%;
  
  &:hover {
    transform: translateY(-5px);
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.1);
    border-color: $primary-light;
  }
}

.feature-icon {
  color: $primary-color;
}

.resources-section {
  padding: 2rem 0;
}

hr {
  border-color: $border;
}

// Responsive adjustments
@media (max-width: 768px) {
  .hero-section {
    padding: 3rem 1rem 2rem;
  }
  
  .home-logo {
    height: 80px;
  }
  
  .display-4 {
    font-size: 2rem;
  }
  
  .h3 {
    font-size: 1.5rem;
  }
}
</style>
