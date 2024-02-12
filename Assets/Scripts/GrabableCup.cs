using System;
using Unity.Netcode;
using UnityEngine;

[RequireComponent(typeof(Rigidbody))]
public class GrabableCup : InteractiveObject
{
    private Rigidbody rb;
    private float startTime = -1f;
    [SerializeField] private float shakeAmplitude = 1f;
    [SerializeField] private float shakeFrequency = 1f;

    protected override void Awake()
    {
        base.Awake();
        rb = GetComponent<Rigidbody>();
    }

    public override bool CanInteract()
    {
        return true;
    }

    protected override void OnInteract()
    {
        rb.useGravity = false;
        rb.constraints = RigidbodyConstraints.FreezeRotation;
        startTime = Time.time;
    }

    protected override void OnEndInteraction()
    {
        rb.useGravity = true;
        rb.constraints = RigidbodyConstraints.None;
        startTime = -1f;
    }

    public Rigidbody GetRigidbody()
    {
        return rb;
    }

    private void FixedUpdate()
    {
        if (startTime > 0f) {
            Transform target = NetworkManager.Singleton.ConnectedClients[GetIsInteracting().Value].PlayerObject.transform;
            transform.LookAt(target, Vector3.up);
            double rotationValue = shakeAmplitude * Math.Sin(2 * Math.PI * shakeFrequency * (Time.time - startTime));
            transform.rotation = Quaternion.Euler(transform.rotation.eulerAngles.x, transform.rotation.eulerAngles.y, (float)rotationValue);
        }
    }
}
