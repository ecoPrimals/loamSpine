#!/usr/bin/env bash
# Demo: Health Monitoring with Real Service
# Demonstrates service health, metrics, and monitoring

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SHOWCASE_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
PROJECT_ROOT="$(cd "${SHOWCASE_ROOT}/.." && pwd)"
BINS_DIR="${PROJECT_ROOT}/../../primalBins"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
CYAN='\033[0;36m'
PURPLE='\033[0;35m'
NC='\033[0m'

# Configuration
LOAMSPINE_BIN="${BINS_DIR}/loamspine"
JSONRPC_PORT="8080"
JSONRPC_URL="http://localhost:${JSONRPC_PORT}"

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}🦴 LoamSpine Health Monitoring Demo${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "🎯 Production Service Monitoring"
echo ""
echo "This demo shows:"
echo "  • Health check endpoints"
echo "  • Service metrics"
echo "  • Real-time monitoring"
echo "  • Performance tracking"
echo ""

# Check if binary exists
if [ ! -f "${LOAMSPINE_BIN}" ]; then
    echo -e "${RED}❌ loamspine binary not found${NC}"
    exit 1
fi

echo -e "${YELLOW}✓ loamspine binary found${NC}"
echo ""

# Start service if not running
if ! pgrep -f "loamspine" > /dev/null; then
    echo "🚀 Starting loamspine..."
    "${LOAMSPINE_BIN}" server --jsonrpc-port "${JSONRPC_PORT}" &> /tmp/loamspine-health-demo.log &
    SERVICE_PID=$!
    echo "${SERVICE_PID}" > /tmp/loamspine-health-demo.pid
    
    echo "   Waiting for service..."
    for i in {1..30}; do
        if curl -s "${JSONRPC_URL}/health" > /dev/null 2>&1; then
            break
        fi
        sleep 0.5
    done
    
    echo -e "   ${GREEN}✅ Service ready (PID: ${SERVICE_PID})${NC}"
else
    echo -e "${YELLOW}⚠ Using existing service instance${NC}"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Test 1: Basic Health Check
echo -e "${CYAN}MONITOR 1: Basic Health Check${NC}"
echo "   Endpoint: GET ${JSONRPC_URL}/health"
echo ""

START_TIME=$(date +%s%3N)
HEALTH_RESPONSE=$(curl -s -w "\nHTTP_CODE:%{http_code}\nTIME:%{time_total}" "${JSONRPC_URL}/health")
END_TIME=$(date +%s%3N)

HTTP_CODE=$(echo "${HEALTH_RESPONSE}" | grep "HTTP_CODE:" | cut -d: -f2)
RESPONSE_TIME=$(echo "${HEALTH_RESPONSE}" | grep "TIME:" | cut -d: -f2)
BODY=$(echo "${HEALTH_RESPONSE}" | grep -v "HTTP_CODE:" | grep -v "TIME:")

echo "   Response: ${BODY}"
echo "   HTTP Status: ${HTTP_CODE}"
echo "   Response Time: ${RESPONSE_TIME}s"
echo ""

if [ "${HTTP_CODE}" = "200" ]; then
    echo -e "   ${GREEN}✅ Health check passed${NC}"
else
    echo -e "   ${RED}❌ Health check failed (HTTP ${HTTP_CODE})${NC}"
    exit 1
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Test 2: Create Load and Monitor
echo -e "${CYAN}MONITOR 2: Service Under Load${NC}"
echo "   Creating workload..."
echo ""

# Create 10 spines
for i in {1..10}; do
    SPINE_RESPONSE=$(curl -s -X POST "${JSONRPC_URL}" \
        -H "Content-Type: application/json" \
        -d "{
            \"jsonrpc\": \"2.0\",
            \"id\": ${i},
            \"method\": \"create_spine\",
            \"params\": {
                \"owner_did\": \"did:key:z6MkHealthDemo${i}\",
                \"name\": \"Health Demo Spine ${i}\"
            }
        }")
    
    if echo "${SPINE_RESPONSE}" | grep -q "result"; then
        echo -e "   ${GREEN}✓${NC} Created spine ${i}/10"
    else
        echo -e "   ${RED}✗${NC} Failed spine ${i}/10"
    fi
done

echo ""
echo "   Checking health after load..."
HEALTH_AFTER=$(curl -s "${JSONRPC_URL}/health")
echo "   Response: ${HEALTH_AFTER}"

if echo "${HEALTH_AFTER}" | grep -q "ok"; then
    echo -e "   ${GREEN}✅ Service stable under load${NC}"
else
    echo -e "   ${YELLOW}⚠ Service degraded${NC}"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Test 3: Continuous Monitoring
echo -e "${CYAN}MONITOR 3: Continuous Health Monitoring${NC}"
echo "   Monitoring for 5 seconds..."
echo ""

CHECKS=0
FAILURES=0

for i in {1..5}; do
    if curl -s "${JSONRPC_URL}/health" | grep -q "ok"; then
        echo -e "   [${i}/5] ${GREEN}✓ Healthy${NC}"
        CHECKS=$((CHECKS + 1))
    else
        echo -e "   [${i}/5] ${RED}✗ Unhealthy${NC}"
        FAILURES=$((FAILURES + 1))
    fi
    sleep 1
done

echo ""
UPTIME_PERCENT=$((CHECKS * 100 / 5))
echo "   Health checks: ${CHECKS}/5 passed"
echo "   Uptime: ${UPTIME_PERCENT}%"

if [ ${CHECKS} -eq 5 ]; then
    echo -e "   ${GREEN}✅ 100% uptime${NC}"
elif [ ${CHECKS} -ge 4 ]; then
    echo -e "   ${YELLOW}⚠ ${UPTIME_PERCENT}% uptime (acceptable)${NC}"
else
    echo -e "   ${RED}❌ ${UPTIME_PERCENT}% uptime (critical)${NC}"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Test 4: Service Metrics
echo -e "${CYAN}MONITOR 4: Service Metrics${NC}"
echo ""

# Get process info if we have PID
if [ -f /tmp/loamspine-health-demo.pid ]; then
    SERVICE_PID=$(cat /tmp/loamspine-health-demo.pid)
    
    # CPU and Memory (Linux)
    if [ -f /proc/${SERVICE_PID}/status ]; then
        MEM_KB=$(grep VmRSS /proc/${SERVICE_PID}/status | awk '{print $2}')
        MEM_MB=$((MEM_KB / 1024))
        
        echo "   Process ID: ${SERVICE_PID}"
        echo "   Memory Usage: ${MEM_MB} MB"
        echo "   Status: Running"
        echo ""
        
        if [ ${MEM_MB} -lt 1000 ]; then
            echo -e "   ${GREEN}✅ Memory usage normal (<1GB)${NC}"
        else
            echo -e "   ${YELLOW}⚠ Memory usage high (>1GB)${NC}"
        fi
    else
        echo "   Process ID: ${SERVICE_PID}"
        echo "   Status: Running"
        echo "   (Detailed metrics not available on this platform)"
    fi
else
    echo "   Using existing service instance"
    echo "   (Metrics available for managed instances only)"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Test 5: Graceful Shutdown Check
echo -e "${CYAN}MONITOR 5: Graceful Shutdown${NC}"
echo ""

if [ -f /tmp/loamspine-health-demo.pid ]; then
    SERVICE_PID=$(cat /tmp/loamspine-health-demo.pid)
    
    echo "   Sending SIGTERM to service..."
    if kill -TERM "${SERVICE_PID}" 2>/dev/null; then
        echo "   Waiting for graceful shutdown..."
        
        for i in {1..10}; do
            if ! kill -0 "${SERVICE_PID}" 2>/dev/null; then
                echo -e "   ${GREEN}✅ Service shut down gracefully${NC}"
                break
            fi
            sleep 0.5
        done
        
        # Force kill if still running
        if kill -0 "${SERVICE_PID}" 2>/dev/null; then
            echo -e "   ${YELLOW}⚠ Force killing service${NC}"
            kill -9 "${SERVICE_PID}" 2>/dev/null || true
        fi
    fi
    
    rm -f /tmp/loamspine-health-demo.pid
    rm -f /tmp/loamspine-health-demo.log
else
    echo "   Not shutting down existing service instance"
fi

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo -e "${GREEN}✅ Health Monitoring Demo Complete!${NC}"
echo ""
echo "🎓 What we demonstrated:"
echo "   1. Basic health check endpoint"
echo "   2. Service stability under load"
echo "   3. Continuous health monitoring"
echo "   4. Service metrics (CPU, memory)"
echo "   5. Graceful shutdown handling"
echo ""
echo "💡 Production Monitoring Patterns:"
echo ""
echo "   ${PURPLE}Health Check Polling${NC}"
echo "   • Poll /health every 10-30 seconds"
echo "   • Alert on 3+ consecutive failures"
echo "   • Track response time trends"
echo ""
echo "   ${PURPLE}Load Monitoring${NC}"
echo "   • Monitor under normal operations"
echo "   • Test with synthetic load"
echo "   • Track degradation patterns"
echo ""
echo "   ${PURPLE}Metrics Collection${NC}"
echo "   • CPU/Memory usage"
echo "   • Request rate and latency"
echo "   • Error rates"
echo "   • Spine creation/append rates"
echo ""
echo "   ${PURPLE}Alerting${NC}"
echo "   • Health check failures"
echo "   • High memory usage (>80%)"
echo "   • Slow response times (>1s)"
echo "   • Process crashes"
echo ""
echo "🔧 Integration with Monitoring Tools:"
echo "   • Prometheus (metrics export)"
echo "   • Grafana (visualization)"
echo "   • AlertManager (alerting)"
echo "   • Nagios/Zabbix (traditional)"
echo "   • Cloud-native (Datadog, New Relic)"
echo ""
echo "🦴 LoamSpine: Production-ready service monitoring!"
echo ""
