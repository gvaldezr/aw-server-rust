# ActivityWatch Server - Quick Start Guide

## 🚀 Inicio Rápido

Este documento proporciona instrucciones para iniciar y acceder al servidor ActivityWatch con PostgreSQL.

---

## 📋 Requisitos Previos

- Docker y Docker Compose instalados
- Puerto 5432 (PostgreSQL), 5600 (API), y 8080 (WebUI) disponibles
- Red privada/LAN para acceso desde múltiples computadoras

---

## 🏃 Iniciar el Sistema

### 1. Iniciar todos los servicios

```bash
docker compose up -d
```

### 2. Verificar estado de servicios

```bash
docker compose ps
```

**Salida esperada:**
```
NAME                       STATUS              PORTS
activitywatch-postgresql   Up (healthy)        0.0.0.0:5432->5432/tcp
activitywatch-server       Up (healthy)        0.0.0.0:5600->5600/tcp
activitywatch-webui        Up (healthy)        0.0.0.0:8080->80/tcp
```

### 3. Verificar logs (opcional)

```bash
# Ver logs de todos los servicios
docker compose logs -f

# Ver logs de un servicio específico
docker compose logs -f aw-server
```

---

## 🌐 Acceso al Sistema

### Acceso Local (desde el servidor)

- **Web UI**: http://localhost:8080
- **API**: http://localhost:5600/api/0/info
- **PostgreSQL**: localhost:5432 (interno)

### Acceso desde Red (desde otras computadoras)

#### 1. Obtener IP del servidor

```bash
# En el servidor, ejecutar:
ifconfig | grep "inet " | grep -v 127.0.0.1
```

**Ejemplo de salida:**
```
inet 192.168.1.50 netmask 0xffffff00 broadcast 192.168.1.255
```

En este ejemplo, la IP es **192.168.1.50**

#### 2. Acceder desde otra computadora

Reemplaza `192.168.1.50` con la IP real de tu servidor:

- **Web UI**: http://192.168.1.50:8080
- **API**: http://192.168.1.50:5600/api/0/info
- **Configurar Watchers**: http://192.168.1.50:5600

---

## 🔧 Configurar Watchers

### aw-watcher-window (Monitoreo de ventanas activas)

Edita el archivo de configuración del watcher:

**Linux/macOS**: `~/.config/activitywatch/aw-watcher-window/config.toml`  
**Windows**: `%APPDATA%\activitywatch\aw-watcher-window\config.toml`

```toml
[server]
hostname = "192.168.1.50"  # IP del servidor
port = 5600
```

### aw-watcher-afk (Monitoreo de ausencia)

Edita el archivo de configuración:

**Linux/macOS**: `~/.config/activitywatch/aw-watcher-afk/config.toml`  
**Windows**: `%APPDATA%\activitywatch\aw-watcher-afk\config.toml`

```toml
[server]
hostname = "192.168.1.50"  # IP del servidor
port = 5600
```

---

## ✅ Verificación de Funcionamiento

### 1. Verificar API

```bash
curl http://192.168.1.50:5600/api/0/info
```

**Respuesta esperada:**
```json
{
  "hostname": "dfbd3f64995b",
  "version": "v0.14.0 (rust)",
  "testing": false,
  "device_id": "05ddfa40-f557-4162-be90-8432d581069d"
}
```

### 2. Verificar Web UI

Abrir en navegador: http://192.168.1.50:8080

Deberías ver la interfaz de ActivityWatch.

### 3. Verificar buckets (después de configurar watchers)

```bash
curl http://192.168.1.50:5600/api/0/buckets
```

---

## 🔒 Seguridad y CORS

### Configuración Actual

**Por defecto**, el servidor permite acceso desde **cualquier origen** (AllowedOrigins::all()), lo cual es apropiado para:
- Redes privadas/internas
- LANs corporativas
- 300+ watchers concurrentes

### Restricción de Acceso (Opcional)

Si necesitas restringir acceso a IPs específicas, edita:

`~/.config/activitywatch/aw-server-rust/config.toml`

```toml
# Permitir solo IPs específicas
cors = [
    "http://192.168.1.100:8080",
    "http://192.168.1.101:8080"
]

# O usar expresiones regulares
cors_regex = [
    "http://192\\.168\\.1\\..*:8080"  # Toda la subred 192.168.1.x
]
```

**Nota**: Cuando configuras `cors` o `cors_regex`, el servidor cambia a modo restrictivo automáticamente.

---

## 📊 Administración

### Detener servicios

```bash
docker compose down
```

### Reiniciar un servicio específico

```bash
docker compose restart aw-server
```

### Ver estadísticas de base de datos

```bash
docker compose exec postgresql psql -U aw_user -d activitywatch -c "\dt+"
```

### Backup de base de datos

```bash
./scripts/backup-database.sh
```

Los backups se guardan en `backups/` con retención de 30 días.

### Restaurar base de datos

```bash
./scripts/restore-database.sh backups/activitywatch-backup-YYYY-MM-DD.sql.gz
```

---

## 🐛 Troubleshooting

### El servicio no inicia

```bash
# Verificar logs
docker compose logs aw-server

# Verificar puertos disponibles
netstat -tuln | grep -E "5432|5600|8080"
```

### Error de conexión CORS desde navegador

Verifica que:
1. El servidor esté escuchando en 0.0.0.0 (no solo 127.0.0.1)
2. Los puertos estén expuestos correctamente en docker-compose.yml
3. No haya firewall bloqueando los puertos

### PostgreSQL no conecta

```bash
# Verificar que PostgreSQL esté saludable
docker compose ps postgresql

# Verificar logs de PostgreSQL
docker compose logs postgresql

# Verificar conectividad
docker compose exec postgresql pg_isready -U aw_user
```

### Web UI muestra página en blanco

```bash
# Verificar logs de nginx
docker compose logs aw-webui

# Verificar que el proxy esté funcionando
curl -I http://localhost:8080/api/0/info
```

---

## 📚 Documentación Adicional

- **README.md**: Documentación completa del proyecto
- **aidlc-docs/construction/unit-1-database-layer/code/deployment-guide.md**: Guía de despliegue detallada
- **aidlc-docs/construction/unit-1-database-layer/COMPLETION-SUMMARY.md**: Resumen de implementación

---

## 🎯 Capacidad del Sistema

Este sistema está configurado para soportar:

- **300 watchers concurrentes**
- **50 eventos/segundo sostenido**
- **300 eventos/segundo pico**
- **1.1 billones eventos/año** (150 GB de datos)
- **Retención de 5 años** (5.5 billones eventos, 750 GB)

---

## 🆘 Soporte

Para problemas o preguntas:
1. Revisa los logs: `docker compose logs -f`
2. Consulta la documentación en `aidlc-docs/`
3. Verifica el estado de salud: `docker compose ps`

---

**¡Sistema listo para producción con 300 watchers! 🚀**
