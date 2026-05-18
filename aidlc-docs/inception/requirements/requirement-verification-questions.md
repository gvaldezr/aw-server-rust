# Requirement Verification Questions

Por favor responde las siguientes preguntas para clarificar los detalles de implementación. Completa cada respuesta después de la etiqueta `[Answer]:`.

---

## Section 1: PostgreSQL Migration

### Q1. PostgreSQL Version & Deployment
¿Cuál es tu preferencia para la versión de PostgreSQL y su despliegue?

A) PostgreSQL 15 (LTS) - versión recomendada, soporte extendido  
B) PostgreSQL 16 - versión más reciente  
C) PostgreSQL 14 - compatibilidad con sistemas antiguos  
D) Otra versión específica  

[Answer]: a

---

### Q2. Database Credentials & Security
¿Cómo deseas manejar las credenciales de PostgreSQL?

A) Variables de entorno (.env file) - recomendado para desarrollo y producción  
B) Hardcoded en config.toml - simple pero menos seguro  
C) Secrets management (Docker Secrets o similar) - para producción segura  
D) Otra alternativa  

[Answer]: a

---

### Q3. Connection String Format
¿Cómo deseas especificar la conexión a PostgreSQL en el servidor?

A) Full connection string en config.toml: `postgresql://user:pass@host:port/db`  
B) Componentes separados: host, port, user, password, database en config.toml  
C) Variables de entorno separadas: `DB_HOST`, `DB_USER`, `DB_PASSWORD`, etc.  
D) Otra configuración  

[Answer]: c

---

### Q4. Data Migration from SQLite
¿Necesitas migrar datos existentes de la instancia SQLite actual?

A) Sí, incluir herramienta de migración automática  
B) No, comenzar con base de datos vacía  
C) Tal vez - proporcionar script de migración opcional  

[Answer]: b

---

## Section 2: Network Configuration

### Q5. Server Binding Address
¿En qué dirección deseas que escuche el servidor?

A) 0.0.0.0 (todas las interfaces) - recomendado para producción/contenedores  
B) Configurable en config.toml, default a 127.0.0.1  
C) Configurable en config.toml, default a 0.0.0.0  
D) Otra opción  

[Answer]: a

---

### Q6. Port Configuration
¿Qué puerto(s) deseas usar?

A) Mantener puertos actuales: 5600 (producción), 5666 (testing)  
B) Cambiar a puertos estándar: 5000 (desarrollo), 8000 (producción)  
C) Configurable por variable de entorno  
D) Otra configuración  

[Answer]: a

---

## Section 3: Docker Deployment

### Q7. Docker Compose Services
¿Cuáles servicios deseas incluir en docker-compose.yml?

A) PostgreSQL + aw-server + aw-webui (completo)  
B) PostgreSQL + aw-server solo (sin UI)  
C) Solo aw-server + aw-webui (assume PostgreSQL externo)  
D) Otra combinación  

[Answer]: a

---

### Q8. Data Persistence in Docker
¿Cómo deseas gestionar la persistencia de datos en Docker?

A) Docker named volumes para PostgreSQL y datos del servidor  
B) Bind mounts (host file system) para fácil acceso  
C) Ambos - volumes nombrados + bind mounts opcionales  
D) Otra estrategia  

[Answer]: a

---

### Q9. WebUI Configuration
¿Cómo deseas que se sirva aw-webui?

A) Desde el mismo contenedor aw-server (embedded static assets)  
B) En contenedor separado (nginx o node server)  
C) Ambos soportados (opción en compose)  

[Answer]: b

---

### Q10. Environment Configuration
¿Cómo deseas pasar configuración al docker-compose?

A) .env file en el directorio raíz (variables de entorno para compose)  
B) Valores hardcoded en docker-compose.yml  
C) docker-compose.override.yml para valores personalizados  
D) Otra alternativa  

[Answer]: b

---

## Section 4: Non-Functional Requirements

### Q11. Database Performance
¿Cuáles son tus expectativas de rendimiento?

A) Optimizar para lectura (muchas queries, pocas escrituras) - actividades de usuarios  
B) Equilibrio lectura/escritura  
C) Alto volumen de escritura (muchos eventos concurrentes)  

[Answer]: b

---

### Q12. Backwards Compatibility
¿Necesitas mantener compatibilidad con la API actual?

A) Sí, API debe ser idéntica (0% cambios en endpoints)  
B) Sí, con deprecaciones gradualmente  
C) No, permitir cambios en API si mejoran design  

[Answer]: a

---

### Q13. Testing Strategy
¿Cómo deseas validar la migración?

A) Tests unitarios + tests de integración automatizados  
B) Solo tests de integración  
C) Tests manuales suficientes  

[Answer]: a

---

## Section 5: Deployment Context

### Q14. Production Deployment Scenario
¿Cuál es el escenario principal de despliegue?

A) Single-machine Docker Compose (un servidor, un contenedor por servicio)  
B) Multi-machine deployment (múltiples servidores)  
C) Kubernetes ready  

[Answer]: a

---

### Q15. Logging & Monitoring
¿Necesitas logging centralizado?

A) Logs a stdout (estándar Docker)  
B) Logs a archivos con rotación  
C) Integración con sistema de logging externo (ELK, Datadog, etc.)  

[Answer]: a

---

## FINAL VALIDATION

Por favor revisa tus respuestas y confirma:

**¿Están todas las respuestas completas y precisas?**

[Answer]: Sí / No / Necesito hacer cambios

**Si hay cambios, ¿cuáles?**

[Answer]: 

---

**Una vez que hayas completado todas las respuestas, confirma con "LISTO".**
