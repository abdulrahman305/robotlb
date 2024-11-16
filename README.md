# Hetzner load balancer tracker

This project is used in cases when you have a bare-metal kubernetes clsuter
and you agents are deployed as dedicated servers.


# Configuration

```yaml
apiVersion: v1
kind: Service
metadata:
  name: target
  annotations:
    # Name of the target loadbalancer
    lb-tracker/balancer: "lb-1"
    # Node selector for NodePort type of service.
    # This selector will select ips of nodes to which loadbalancer traffic will be routed.
    # All nodes included if label is not present.
    lb-tracker/node-selector: "node-role.kubernetes.io/control-plane!=true,beta.kubernetes.io/arch=amd64"
    # Load balancer healthcheck options
    lb-tracker/lb-check-interval: "1"
    lb-tracker/lb-timeout: "10"
    lb-tracker/lb-retries: "3"
    # Wether to use proxymode for this target.
    # https://docs.hetzner.com/cloud/load-balancers/faq/#what-does-proxy-protocol-mean-and-should-i-enable-it
    lb-tracker/lb-proxy-mode: "true"
spec:
  type: NodePort
  selector:
    app: target
  ports:
    - protocol: TCP
      port: 80         # This will be listening_port on LB
      nodePort: 33744  # This will become a target port
      targetPort: 80
```
