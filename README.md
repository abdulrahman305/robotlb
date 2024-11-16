# Hetzner load balancer tracker

This project is used in cases when you have a bare-metal kubernetes clsuter
and you agents are deployed as dedicated servers.


# Configuration

```yaml
apiVersion: v1
kind: Service
metadata:
  name: target
spec:
  selector:
    app: target
  ports:
    - protocol: TCP
      port: 80
      targetPort: 80
```
