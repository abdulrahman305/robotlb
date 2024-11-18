# Hetzner bare-metal load balancer

This project can be used when you have deployed a bare-metal kubernetes cluster on
hetzner robot and want to use hetzner's load-balaner.

This small operator will do it for you.


# Prerequisites

Before you can use this operator, please make sure:

1. You have a cluster deployed on [Hetzner robot](https://robot.hetzner.com/) (at least agent nodes);
2. You created a [vSwitch](https://docs.hetzner.com/robot/dedicated-server/network/vswitch/) for these servers;
3. You have a cloud network with subnet that points to the vSwitch ([Tutorial](https://docs.hetzner.com/cloud/networks/connect-dedi-vswitch/));

If you have all the requirements met, you can create a service of type LoadBalancer.

# Configuration

```yaml
apiVersion: v1
kind: Service
metadata:
  name: target
  annotations:
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
    lb-tracker/lb-proxy-mode: "false"
spec:
  type: LoadBalancer
  selector:
    app: target
  ports:
    - protocol: TCP
      port: 80         # This will be listening_port on the LB
      targetPort: 80
```

## Configuration
